//! Polymarket CLOB API 客户端 - 使用官方 SDK 封装

use alloy::primitives::ChainId;
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use anyhow::{Context, Result};
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::clob::types::{
    request::CancelMarketOrderRequest, request::UpdateBalanceAllowanceRequest, AssetType,
    OrderType as SdkOrderType, Side as SdkSide,
};
use polymarket_client_sdk::clob::{Client as ClobSdkClient, Config};
use polymarket_client_sdk::types::U256;
use std::str::FromStr;
use tracing::instrument;

use super::types::*;

const POLYGON_CHAIN_ID: ChainId = 137;

pub struct ClobClient {
    host: String,
    private_key: Option<String>,
    credentials: Option<Credentials>,
    proxy_url: Option<String>,
}

impl ClobClient {
    pub fn new(
        host: &str,
        private_key: Option<String>,
        credentials: Option<Credentials>,
        proxy_url: Option<String>,
    ) -> Self {
        Self {
            host: host.trim_end_matches('/').to_string(),
            private_key,
            credentials,
            proxy_url,
        }
    }

    #[instrument(skip(self))]
    pub async fn get_orderbook(&self, token_id: &str) -> Result<OrderBookResponse> {
        use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;

        let client = ClobSdkClient::new(&self.host, self.sdk_config())
            .context("Failed to create SDK client")?;

        let token_id_u256 =
            U256::from_str_radix(token_id, 10).context("Failed to parse token_id")?;

        let request = OrderBookSummaryRequest::builder()
            .token_id(token_id_u256)
            .build();

        let book = client
            .order_book(&request)
            .await
            .context("Failed to fetch orderbook")?;

        let bids: Vec<Level> = book
            .bids
            .iter()
            .map(|level| Level {
                price: level.price,
                size: level.size,
            })
            .collect();

        let asks: Vec<Level> = book
            .asks
            .iter()
            .map(|level| Level {
                price: level.price,
                size: level.size,
            })
            .collect();

        Ok(OrderBookResponse {
            token_id: token_id.to_string(),
            bids,
            asks,
            timestamp: book.timestamp.timestamp() as u64,
        })
    }

    #[instrument(skip(self))]
    pub async fn place_order(&self, request: OrderRequest) -> Result<OrderResponse> {
        let sdk_client = self.authenticated_client().await?;

        sdk_client
            .update_balance_allowance(
                UpdateBalanceAllowanceRequest::builder()
                    .asset_type(AssetType::Collateral)
                    .build(),
            )
            .await
            .context("Failed to update balance allowance")?;
        tracing::info!("Successfully updated balance/allowance cache");

        let token_id =
            U256::from_str_radix(&request.token_id, 10).context("Failed to parse token_id")?;

        let tick_size = sdk_client
            .tick_size(token_id)
            .await
            .context("Failed to fetch tick size")?
            .minimum_tick_size
            .as_decimal();
        let decimals = tick_size.scale();

        let price = request.price.round_dp(decimals);
        let size = request.size.round_dp(2);

        let order = sdk_client
            .limit_order()
            .token_id(token_id)
            .side(match request.side {
                Side::Buy => SdkSide::Buy,
                Side::Sell => SdkSide::Sell,
            })
            .price(price)
            .size(size)
            .order_type(match request.order_type {
                OrderType::Gtc => SdkOrderType::GTC,
                OrderType::Fok => SdkOrderType::FOK,
                OrderType::Ioc => SdkOrderType::FAK,
            })
            .build()
            .await
            .context("Failed to build order")?;

        let signer = self.signer()?;
        let signed_order = sdk_client
            .sign(&signer, order)
            .await
            .context("Failed to sign order")?;

        let post_result = sdk_client.post_order(signed_order).await;
        match &post_result {
            Ok(resp) => {
                tracing::info!("Order submitted successfully: order_id={}", resp.order_id);
            }
            Err(err) => {
                tracing::error!("Order submission failed: {:?}", err);
                let err_str = format!("{:?}", err);
                if err_str.contains("status") {
                    tracing::error!("HTTP error details: {}", err_str);
                }
            }
        }
        let response = post_result.context("Failed to submit order")?;

        Ok(OrderResponse {
            success: true,
            order_id: response.order_id.to_string(),
            signature: None,
        })
    }

    #[instrument(skip(self))]
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse> {
        let sdk_client = self.authenticated_client().await?;
        let response = sdk_client.cancel_orders(&[order_id]).await?;
        let success = !response.canceled.is_empty();

        Ok(CancelOrderResponse {
            success,
            order_id: order_id.to_string(),
        })
    }

    #[instrument(skip(self))]
    pub async fn cancel_all(&self, market: Option<&str>) -> Result<Vec<String>> {
        let sdk_client = self.authenticated_client().await?;

        let cancelled = if let Some(market_hash) = market {
            let market_b256 = market_hash.parse::<alloy::primitives::B256>()?;
            let request = CancelMarketOrderRequest::builder()
                .market(market_b256)
                .build();
            sdk_client.cancel_market_orders(&request).await?
        } else {
            sdk_client.cancel_all_orders().await?
        };

        Ok(cancelled.canceled.iter().map(|id| id.to_string()).collect())
    }

    async fn authenticated_client(
        &self,
    ) -> Result<
        ClobSdkClient<
            polymarket_client_sdk::auth::state::Authenticated<polymarket_client_sdk::auth::Normal>,
        >,
    > {
        let signer = self.signer()?;
        let mut builder =
            ClobSdkClient::new(&self.host, self.sdk_config())?.authentication_builder(&signer);

        if let Some(credentials) = &self.credentials {
            builder = builder.credentials(credentials.clone());
        }

        builder
            .authenticate()
            .await
            .context("Failed to authenticate")
    }

    fn sdk_config(&self) -> Config {
        if let Some(proxy_url) = &self.proxy_url {
            Config::builder().proxy_url(proxy_url.clone()).build()
        } else {
            Config::default()
        }
    }

    fn signer(&self) -> Result<LocalSigner<alloy::signers::k256::ecdsa::SigningKey>> {
        let private_key = self
            .private_key
            .as_ref()
            .context("Private key not configured")?;
        let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
        LocalSigner::from_str(pk)
            .context("Failed to parse private key")?
            .with_chain_id(Some(POLYGON_CHAIN_ID))
            .pipe(Ok)
    }
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
impl<T> Pipe for T {}
