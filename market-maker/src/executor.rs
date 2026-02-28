//! 订单执行器 - 使用官方 SDK

use anyhow::{Context, Result};
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use polymarket_client_sdk::clob::types::request::{OrderBookSummaryRequest, OrdersRequest};
use polymarket_client_sdk::clob::types::{Side as SdkSide, Amount};
use polymarket_client_sdk::types::{Decimal, U256};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use rust_decimal_macros::dec;
use std::str::FromStr;
use tracing::{info, warn, instrument};

use crate::config::Config;

/// 简化的客户端包装器，隐藏复杂的泛型
pub struct ClobClient {
    inner: polymarket_client_sdk::clob::Client,
}

impl Clone for ClobClient {
    fn clone(&self) -> Self {
        // Client 本身支持 Clone
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl ClobClient {
    pub async fn new(host: &str, private_key: &str) -> Result<Self> {
        let signer = LocalSigner::from_str(private_key)?
            .with_chain_id(Some(POLYGON));

        let client = polymarket_client_sdk::clob::Client::new(host, Default::default())?
            .authentication_builder(&signer)
            .authenticate()
            .await?;

        Ok(Self { inner: client })
    }

    pub async fn order_book(&self, token_id: U256) -> Result<polymarket_client_sdk::clob::types::response::OrderBookSummaryResponse> {
        let request = OrderBookSummaryRequest::builder()
            .token_id(token_id)
            .build();
        
        Ok(self.inner.order_book(&request).await?)
    }

    pub async fn place_limit_order(&self, token_id: U256, price: Decimal, size: Decimal, side: SdkSide) -> Result<String> {
        let order = self.inner
            .limit_order()
            .token_id(token_id)
            .price(price)
            .amount(Amount::usdc(size)?)
            .side(side)
            .build()
            .await?;

        Ok(order.order_id.to_string())
    }

    pub async fn cancel_all(&self, market_id: Option<&str>) -> Result<()> {
        Ok(self.inner.cancel_all(market_id).await?)
    }

    pub async fn orders(&self) -> Result<Vec<String>> {
        let request = OrdersRequest::builder().build();
        let page = self.inner.orders(&request).await?;
        Ok(page.data.into_iter().map(|o| o.order_id.to_string()).collect())
    }
}

#[derive(Clone)]
pub struct Executor {
    client: ClobClient,
    order_size: Decimal,
}

impl Executor {
    pub async fn new(config: &Config) -> Result<Self> {
        let private_key = std::env::var(PRIVATE_KEY_VAR)
            .context("Need POLYMARKET_PRIVATE_KEY environment variable")?;
        
        let client = ClobClient::new(&config.clob.host, &private_key).await?;
        let order_size = Decimal::from_f64_retain(config.strategy.order_size_usd)
            .unwrap_or(dec!(1000));

        Ok(Self {
            client,
            order_size,
        })
    }

    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn fetch_orderbook(&self, token_id: &str) -> Result<polymarket_client_sdk::clob::types::response::OrderBookSummaryResponse> {
        let token_id_sdk = U256::from_str(token_id)?;
        self.client.order_book(token_id_sdk).await
    }

    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn place_orders(&self, token_id: &str, bid_price: f64, ask_price: f64) -> Result<()> {
        let token_id_sdk = U256::from_str(token_id)?;
        let bid_dec = Decimal::from_f64_retain(bid_price).unwrap_or(dec!(0.50));
        let ask_dec = Decimal::from_f64_retain(ask_price).unwrap_or(dec!(0.50));
        let size = self.order_size;

        // 下买单
        match self.client.place_limit_order(token_id_sdk, bid_dec, size, SdkSide::Buy).await {
            Ok(order_id) => {
                info!("Buy order placed: {}", order_id);
            }
            Err(e) => {
                warn!("Failed to place buy order: {}", e);
            }
        }

        // 下卖单
        match self.client.place_limit_order(token_id_sdk, ask_dec, size, SdkSide::Sell).await {
            Ok(order_id) => {
                info!("Sell order placed: {}", order_id);
            }
            Err(e) => {
                warn!("Failed to place sell order: {}", e);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cancel_orders(&self, _market_id: &str) -> Result<()> {
        warn!("Cancel orders not fully implemented");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cancel_all_orders(&self) -> Result<()> {
        self.client.cancel_all(None).await?;
        info!("All orders cancelled");
        Ok(())
    }

    pub async fn get_orders(&self) -> Result<Vec<String>> {
        self.client.orders().await
    }
}
