//! 官方 SDK 适配器

use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::order_builder::LimitParams;
use polymarket_client_sdk::clob::types::request::{
    OrderBookRequest, OrdersRequest, CancelOrderRequest, BalanceRequest,
};
use polymarket_client_sdk::clob::types::{Side as SdkSide, SignatureType};
use futures_util::StreamExt;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::{info, warn};

use crate::auth::AuthConfig;
use crate::error::{Error, Result};
use crate::types::{self, OrderBook, Order, Position, Trade, f64_to_decimal, str_to_token_id};

/// Polymarket 适配器
/// 
/// 封装官方 SDK，提供简化的 API 接口
pub struct PolyAdapter {
    client: Client,
}

impl PolyAdapter {
    /// 创建新的适配器（需要先认证）
    pub async fn new(clob_host: &str, private_key: &str) -> Result<Self> {
        let auth_config = AuthConfig::new(private_key, clob_host);
        let client = auth_config.create_client().await?;
        Ok(Self { client })
    }

    /// 从认证配置创建
    pub async fn from_config(config: &AuthConfig) -> Result<Self> {
        let client = config.create_client().await?;
        Ok(Self { client })
    }

    // ========== 市场数据 ==========

    /// 获取订单簿
    pub async fn get_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        let token_id_sdk = str_to_token_id(token_id)?;
        let request = OrderBookRequest { token_id: token_id_sdk };
        
        let ob = self.client.orderbook(&request).await?;
        
        Ok(OrderBook {
            token_id: token_id.to_string(),
            bids: ob.bids.into_iter().map(|l| types::OrderBookLevel {
                price: l.price,
                size: l.size,
            }).collect(),
            asks: ob.asks.into_iter().map(|l| types::OrderBookLevel {
                price: l.price,
                size: l.size,
            }).collect(),
            timestamp: ob.timestamp as u64,
        })
    }

    /// 获取中间价
    pub async fn get_midpoint(&self, token_id: &str) -> Result<Decimal> {
        use polymarket_client_sdk::clob::types::request::MidpointRequest;
        
        let token_id_sdk = str_to_token_id(token_id)?;
        let request = MidpointRequest { token_id: token_id_sdk };
        
        let resp = self.client.midpoint(&request).await?;
        Ok(resp.mid)
    }

    /// 获取价差
    pub async fn get_spread(&self, token_id: &str) -> Result<Decimal> {
        use polymarket_client_sdk::clob::types::request::SpreadRequest;
        
        let token_id_sdk = str_to_token_id(token_id)?;
        let request = SpreadRequest { token_id: token_id_sdk };
        
        let resp = self.client.spread(&request).await?;
        Ok(resp.spread)
    }

    // ========== 下单 ==========

    /// 下买单（限价单）
    pub async fn buy(&self, token_id: &str, price: f64, size: f64) -> Result<String> {
        self.place_limit_order(token_id, price, size, SdkSide::Buy).await
    }

    /// 下卖单（限价单）
    pub async fn sell(&self, token_id: &str, price: f64, size: f64) -> Result<String> {
        self.place_limit_order(token_id, price, size, SdkSide::Sell).await
    }

    /// 下限价单
    async fn place_limit_order(&self, token_id: &str, price: f64, size: f64, side: SdkSide) -> Result<String> {
        let token_id_sdk = str_to_token_id(token_id)?;
        let price_dec = f64_to_decimal(price)?;
        let size_dec = f64_to_decimal(size)?;

        // 使用官方的订单构建器
        let order = self.client
            .limit_order_builder()
            .token_id(token_id_sdk)
            .price(price_dec)
            .size(size_dec)
            .side(side)
            .build()
            .await?;

        let resp = self.client.create_order(order).await?;
        info!("Order placed: {}", resp.order_id);
        
        Ok(resp.order_id)
    }

    /// 下市价单
    pub async fn market_order(&self, token_id: &str, size: f64, side: SdkSide) -> Result<String> {
        use polymarket_client_sdk::clob::order_builder::MarketParams;
        
        let token_id_sdk = str_to_token_id(token_id)?;
        let size_dec = f64_to_decimal(size)?;

        let order = self.client
            .market_order_builder()
            .token_id(token_id_sdk)
            .size(size_dec)
            .side(side)
            .build()
            .await?;

        let resp = self.client.create_order(order).await?;
        info!("Market order placed: {}", resp.order_id);
        
        Ok(resp.order_id)
    }

    // ========== 订单管理 ==========

    /// 取消订单
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let request = CancelOrderRequest {
            order_id: uuid::Uuid::from_str(order_id)?,
        };
        
        self.client.cancel(&request).await?;
        info!("Order cancelled: {}", order_id);
        Ok(())
    }

    /// 取消所有订单
    pub async fn cancel_all(&self, market_id: Option<&str>) -> Result<()> {
        self.client.cancel_all(market_id).await?;
        info!("All orders cancelled");
        Ok(())
    }

    /// 获取用户订单
    pub async fn get_orders(&self, market_id: Option<&str>) -> Result<Vec<Order>> {
        let request = OrdersRequest {
            id: None,
            user: None,
            market: market_id.map(|s| s.to_string()),
            asset_id: None,
            side: None,
            status: None,
        };

        let page = self.client.orders(&request).await?;
        
        Ok(page.data.into_iter().map(|o| Order {
            order_id: o.order_id.to_string(),
            token_id: o.asset_id.to_string(),
            price: o.price,
            size: o.size,
            side: o.side,
            status: o.status,
        }).collect())
    }

    // ========== 用户数据 ==========

    /// 获取持仓
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        use polymarket_client_sdk::data::Client as DataClient;
        
        // Data API 需要单独初始化
        let data_client = DataClient::default();
        let positions = data_client.user_positions(&Default::default()).await?;
        
        Ok(positions.into_iter().map(|p| Position {
            token_id: p.token_id.to_string(),
            balance: p.balance,
            total_cost: p.total_cost,
        }).collect())
    }

    /// 获取余额
    pub async fn get_balance(&self, token_id: &str) -> Result<Decimal> {
        let token_id_sdk = str_to_token_id(token_id)?;
        let request = BalanceRequest { token_id: token_id_sdk };
        
        let resp = self.client.balance(&request).await?;
        Ok(resp.balance)
    }

    /// 获取交易记录
    pub async fn get_trades(&self, limit: u32) -> Result<Vec<Trade>> {
        use polymarket_client_sdk::data::Client as DataClient;
        use polymarket_client_sdk::data::types::request::UserTradesRequest;
        
        let data_client = DataClient::default();
        let request = UserTradesRequest {
            id: None,
            user: None,
            market: None,
            asset_id: None,
            side: None,
            limit: Some(limit as usize),
        };

        let trades = data_client.user_trades(&request).await?;
        
        Ok(trades.into_iter().map(|t| Trade {
            order_id: t.order_id.to_string(),
            token_id: t.asset_id.to_string(),
            side: t.side,
            price: t.price,
            size: t.size,
            fee: t.fee,
            timestamp: t.timestamp as u64,
        }).collect())
    }

    // ========== WebSocket ==========

    /// 订阅订单簿更新
    pub async fn subscribe_orderbook(
        &self,
        token_ids: Vec<String>,
    ) -> Result<impl futures_util::Stream<Item = Result<OrderBook>>> {
        let token_ids_sdk: Vec<_> = token_ids.iter()
            .filter_map(|id| str_to_token_id(id).ok())
            .collect();

        let stream = self.client.subscribe_orderbook(token_ids_sdk).await?;
        
        Ok(stream.then(|result| {
            async move {
                match result {
                    Ok(ob) => Ok(OrderBook {
                        token_id: token_id_to_str(ob.token_id),
                        bids: ob.bids.into_iter().map(|l| types::OrderBookLevel {
                            price: l.price,
                            size: l.size,
                        }).collect(),
                        asks: ob.asks.into_iter().map(|l| types::OrderBookLevel {
                            price: l.price,
                            size: l.size,
                        }).collect(),
                        timestamp: ob.timestamp as u64,
                    }),
                    Err(e) => Err(Error::Sdk(e)),
                }
            }
        }))
    }
}

// 辅助函数
use crate::types::token_id_to_str;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要真实 API 密钥
    async fn test_get_orderbook() {
        // 这个测试需要真实的 API 密钥
        // 在实际环境中运行
    }
}
