//! Polymarket CLOB API 客户端 - 使用官方 SDK 封装

use anyhow::{Context, Result};
use polymarket_client_sdk::clob::{Client as ClobSdkClient, Config};
use polymarket_client_sdk::clob::types::{
    Side as SdkSide, OrderType as SdkOrderType, request::CancelMarketOrderRequest,
    request::UpdateBalanceAllowanceRequest, AssetType,
};
use polymarket_client_sdk::types::U256;
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use std::str::FromStr;
use alloy::primitives::ChainId;
use tracing::instrument;

use super::types::*;

const POLYGON_CHAIN_ID: ChainId = 137;

/// CLOB API 客户端
pub struct ClobClient {
    host: String,
    private_key: Option<String>,
}

impl ClobClient {
    /// 创建新的客户端
    pub fn new(host: &str, private_key: Option<String>) -> Self {
        Self {
            host: host.trim_end_matches('/').to_string(),
            private_key,
        }
    }

    /// 获取订单簿 (公开端点)
    #[instrument(skip(self))]
    pub async fn get_orderbook(&self, token_id: &str) -> Result<OrderBookResponse> {
        use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;
        
        let client = ClobSdkClient::new(&self.host, Config::default())
            .context("Failed to create SDK client")?;
        
        let token_id_u256 = U256::from_str_radix(token_id, 10)
            .context("Failed to parse token_id")?;
        
        let request = OrderBookSummaryRequest::builder()
            .token_id(token_id_u256)
            .build();
        
        let book = client.order_book(&request).await
            .context("Failed to fetch orderbook")?;
        
        let bids: Vec<Level> = book.bids.iter().map(|l| Level {
            price: l.price,
            size: l.size,
        }).collect();
        
        let asks: Vec<Level> = book.asks.iter().map(|l| Level {
            price: l.price,
            size: l.size,
        }).collect();
        
        Ok(OrderBookResponse {
            token_id: token_id.to_string(),
            bids,
            asks,
            timestamp: book.timestamp.timestamp() as u64,
        })
    }
    
    /// 下单
    #[instrument(skip(self))]
    pub async fn place_order(&self, request: OrderRequest) -> Result<OrderResponse> {
        let private_key = self.private_key.as_ref()
            .context("Private key not configured")?;
        
        // 创建认证客户端
        let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
        let signer = LocalSigner::from_str(pk)
            .context("Failed to parse private key")?
            .with_chain_id(Some(POLYGON_CHAIN_ID));
        
        let sdk_client = ClobSdkClient::new(&self.host, Config::default())?
            .authentication_builder(&signer)
            
            .authenticate()
            .await
            .context("Failed to authenticate")?;
        
        // 更新余额/授权缓存 - 这是 Python SDK 成功的关键步骤
        // 确保 CLOB 后端有最新的链上余额和授权状态
        sdk_client
            .update_balance_allowance(UpdateBalanceAllowanceRequest::builder()
                .asset_type(AssetType::Collateral)
                .build())
            .await
            .context("Failed to update balance allowance")?;
        tracing::info!("Successfully updated balance/allowance cache");
        
        // 构建订单
        let token_id = U256::from_str_radix(&request.token_id, 10)
            .context("Failed to parse token_id")?;
        
        let tick_size = sdk_client.tick_size(token_id).await
            .context("Failed to fetch tick size")?
            .minimum_tick_size
            .as_decimal();
        let decimals = tick_size.scale();
        
        let price = request.price.round_dp(decimals);
        let size = request.size.round_dp(2);
        
        let order_builder = sdk_client.limit_order()
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
                OrderType::Ioc => SdkOrderType::FAK, // IOC 映射到 FAK (Fill and Kill)
            });
        
        // 构建并签名订单
        let order_res = order_builder.build().await;
        if let Err(ref e) = order_res {
            tracing::error!("Order build error details: {:?}", e);
        }
        let order = order_res
            .context("Failed to build order")?;
        
        let signed_order = sdk_client.sign(&signer, order).await
            .context("Failed to sign order")?;
        
        // 提交订单
        let response = sdk_client.post_order(signed_order).await
            .context("Failed to submit order")?;
        
        Ok(OrderResponse {
            success: true,
            order_id: response.order_id.to_string(),
            signature: None,
        })
    }
    
    /// 取消订单
    #[instrument(skip(self))]
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse> {
        let private_key = self.private_key.as_ref()
            .context("Private key not configured")?;
        
        let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
        let signer = LocalSigner::from_str(pk)?
            .with_chain_id(Some(POLYGON_CHAIN_ID));
        
        let sdk_client = ClobSdkClient::new(&self.host, Config::default())?
            .authentication_builder(&signer)
            
            .authenticate()
            .await?;
        
        // 使用 cancel_orders 批量 API 取消单个订单
        let response = sdk_client.cancel_orders(&[order_id]).await?;
        
        let success = !response.canceled.is_empty();
        
        Ok(CancelOrderResponse {
            success,
            order_id: order_id.to_string(),
        })
    }
    
    /// 取消所有订单
    #[instrument(skip(self))]
    pub async fn cancel_all(&self, market: Option<&str>) -> Result<Vec<String>> {
        let private_key = self.private_key.as_ref()
            .context("Private key not configured")?;
        
        let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
        let signer = LocalSigner::from_str(pk)?
            .with_chain_id(Some(POLYGON_CHAIN_ID));
        
        let sdk_client = ClobSdkClient::new(&self.host, Config::default())?
            .authentication_builder(&signer)
            
            .authenticate()
            .await?;
        
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
}
