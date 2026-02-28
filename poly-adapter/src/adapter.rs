//! 官方 SDK 适配器 - 简化版本

use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::request::{
    MidpointRequest, OrderBookSummaryRequest, PriceRequest, SpreadRequest,
};
use polymarket_client_sdk::clob::types::{Side as SdkSide};
use polymarket_client_sdk::types::U256;
use futures_util::StreamExt;
use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::auth::AuthConfig;
use crate::error::{Error, Result};
use crate::types::{self, OrderBook, Order, f64_to_decimal, str_to_token_id, token_id_to_str};

/// Polymarket 适配器
pub struct PolyAdapter {
    client: Client,
}

impl PolyAdapter {
    /// 创建新的适配器
    pub async fn new(clob_host: &str, private_key: &str) -> Result<Self> {
        let auth_config = AuthConfig::new(private_key, clob_host);
        let client = auth_config.create_client().await?;
        Ok(Self { client })
    }

    // ========== 市场数据 ==========

    /// 获取订单簿
    pub async fn get_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        let token_id_sdk = str_to_token_id(token_id)?;
        let request = OrderBookSummaryRequest::builder()
            .token_id(token_id_sdk)
            .build();
        
        let ob = self.client.order_book(&request).await?;
        
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
        let token_id_sdk = str_to_token_id(token_id)?;
        let request = MidpointRequest::builder()
            .token_id(token_id_sdk)
            .build();
        
        let resp = self.client.midpoint(&request).await?;
        Ok(resp.mid)
    }

    /// 获取价差
    pub async fn get_spread(&self, token_id: &str) -> Result<Decimal> {
        let token_id_sdk = str_to_token_id(token_id)?;
        let request = SpreadRequest::builder()
            .token_id(token_id_sdk)
            .build();
        
        let resp = self.client.spread(&request).await?;
        Ok(resp.spread)
    }

    /// 获取价格
    pub async fn get_price(&self, token_id: &str) -> Result<Decimal> {
        let token_id_sdk = str_to_token_id(token_id)?;
        let request = PriceRequest::builder()
            .token_id(token_id_sdk)
            .side(SdkSide::Buy)
            .build();
        
        let resp = self.client.price(&request).await?;
        Ok(resp.price)
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

        let order = self.client
            .limit_order()
            .token_id(token_id_sdk)
            .price(price_dec)
            .size(size_dec)
            .side(side)
            .build()
            .await?;

        let resp = self.client.create_order(&order).await?;
        info!("Order placed: {}", resp.order_id);
        
        Ok(resp.order_id)
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        use uuid::Uuid;
        let order_uuid = Uuid::parse_str(order_id)?;
        self.client.cancel_order(order_uuid).await?;
        info!("Order cancelled: {}", order_id);
        Ok(())
    }

    /// 取消所有订单 - 暂不实现
    pub async fn cancel_all(&self, _market_id: Option<&str>) -> Result<()> {
        warn!("cancel_all not implemented yet");
        Ok(())
    }

    /// 获取用户订单 - 暂不实现
    pub async fn get_orders(&self) -> Result<Vec<Order>> {
        warn!("get_orders not implemented yet");
        Ok(vec![])
    }

    // ========== 简化方法（暂不实现） ==========

    /// 获取持仓 - 暂不实现
    pub async fn get_positions(&self) -> Result<Vec<types::Position>> {
        warn!("get_positions not implemented yet");
        Ok(vec![])
    }

    /// 获取余额 - 暂不实现
    pub async fn get_balance(&self, _token_id: &str) -> Result<Decimal> {
        warn!("get_balance not implemented yet");
        Ok(Decimal::ZERO)
    }

    /// 获取交易记录 - 暂不实现
    pub async fn get_trades(&self, _limit: u32) -> Result<Vec<types::Trade>> {
        warn!("get_trades not implemented yet");
        Ok(vec![])
    }

    /// 下市价单 - 暂不实现
    pub async fn market_order(&self, _token_id: &str, _size: f64, _side: SdkSide) -> Result<String> {
        warn!("market_order not implemented yet");
        Err(Error::Order("Market orders not implemented".to_string()))
    }

    /// WebSocket 订阅 - 暂不实现
    pub async fn subscribe_orderbook(
        &self,
        _token_ids: Vec<String>,
    ) -> Result<impl futures_util::Stream<Item = Result<OrderBook>> + '_> {
        warn!("subscribe_orderbook not implemented yet");
        // 返回空 stream
        Ok(futures_util::stream::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_get_orderbook() {
        // 需要真实 API 密钥
    }
}
