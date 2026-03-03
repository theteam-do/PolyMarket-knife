//! 订单执行器 - 生产级实现

use anyhow::Result;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, instrument, warn};

use crate::api::client::ClobClient;
use crate::api::types::{OrderRequest, OrderType, Side};
use crate::config::Config;
use crate::order_book::{Level, OrderBook};

/// 订单执行器
pub struct Executor {
    order_size: Decimal,
    client: ClobClient,
    nonce: u64,
}

impl Executor {
    /// 创建新的执行器
    pub fn new(config: &Config) -> Result<Self> {
        let order_size = Decimal::from_f64_retain(config.strategy.order_size_usd)
            .unwrap_or(dec!(1000));

        // 创建 CLOB 客户端（使用官方 SDK，私钥用于签名）
        let client = ClobClient::new(&config.clob.host, Some(config.polygon.private_key.clone()));
        info!("CLOB client configured for wallet");

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() * 1000;

        info!("Executor initialized with order_size: ${}", order_size);

        Ok(Self { order_size, client, nonce })
    }

    /// 获取订单簿
    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        info!("Fetching orderbook for token: {}", token_id);
        
        // 调用 CLOB API 获取订单簿
        match self.client.get_orderbook(token_id).await {
            Ok(response) => {
                info!(
                    "Orderbook response token={} ts={} bids={} asks={}",
                    response.token_id,
                    response.timestamp,
                    response.bids.len(),
                    response.asks.len()
                );
                // 转换 API 响应为内部 OrderBook 结构
                let bids: Vec<Level> = response.bids.iter().map(|l| Level {
                    price: l.price.to_string().parse().unwrap_or(0.0),
                    size: l.size.to_string().parse().unwrap_or(0.0),
                }).collect();
                
                let asks: Vec<Level> = response.asks.iter().map(|l| Level {
                    price: l.price.to_string().parse().unwrap_or(0.0),
                    size: l.size.to_string().parse().unwrap_or(0.0),
                }).collect();
                
                let mut order_book = OrderBook::new(token_id.to_string());
                order_book.bids = bids;
                order_book.asks = asks;
                order_book.update_best();
                
                Ok(order_book)
            }
            Err(e) => {
                warn!("Failed to fetch orderbook from API: {}. Using empty book.", e);
                // 返回空订单簿，让做市商继续运行
                Ok(OrderBook {
                    token_id: token_id.to_string(),
                    bids: vec![],
                    asks: vec![],
                    best_bid: None,
                    best_ask: None,
                })
            }
        }
    }

    /// 下双边订单
    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn place_orders(
        &mut self,
        token_id: &str,
        bid_price: f64,
        ask_price: f64,
    ) -> Result<(Option<String>, Option<String>)> {
        let bid_dec = Decimal::from_f64_retain(bid_price).unwrap_or(dec!(0.50));
        let ask_dec = Decimal::from_f64_retain(ask_price).unwrap_or(dec!(0.50));

        info!(
            "Placing orders for {}: bid={}, ask={}, size={}",
            token_id, bid_dec, ask_dec, self.order_size
        );

        // 下买单
        let buy_result = match self
            .place_limit_order(token_id, bid_dec, self.order_size, "BUY")
            .await
        {
            Ok(order_id) => {
                info!("Buy order placed: {}", order_id);
                Some(order_id)
            }
            Err(e) => {
                error!("Failed to place buy order: {}", e);
                None
            }
        };

        // 下卖单
        let sell_result = match self
            .place_limit_order(token_id, ask_dec, self.order_size, "SELL")
            .await
        {
            Ok(order_id) => {
                info!("Sell order placed: {}", order_id);
                Some(order_id)
            }
            Err(e) => {
                error!("Failed to place sell order: {}", e);
                None
            }
        };

        Ok((buy_result, sell_result))
    }

    /// 下限价单
    async fn place_limit_order(
        &mut self,
        token_id: &str,
        price: Decimal,
        size: Decimal,
        side: &str,
    ) -> Result<String> {
        // 生成 nonce
        self.nonce = self.nonce.saturating_add(1);
        let nonce = self.nonce;
        
        // 构建订单请求
        let order_side = match side {
            "BUY" => Side::Buy,
            "SELL" => Side::Sell,
            _ => anyhow::bail!("Invalid side: {}", side),
        };
        
        let request = OrderRequest {
            token_id: token_id.to_string(),
            price,
            size,
            side: order_side,
            order_type: OrderType::Gtc,
            expiration: 0,
            nonce,
            signer: "".to_string(), // 由 client 填充
        };

        info!(
            "Placing {} order: token={}, price={}, size={}, nonce={}",
            side, token_id, price, size, nonce
        );

        // 调用 CLOB API 下单
        match self.client.place_order(request).await {
            Ok(response) => {
                if !response.success {
                    anyhow::bail!("Order rejected by API: {}", response.order_id);
                }
                if let Some(sig) = &response.signature {
                    info!("Order signature received: {}", sig);
                }
                info!("Order placed successfully: {}", response.order_id);
                Ok(response.order_id)
            }
            Err(e) => {
                error!("Failed to place order: {}", e);
                Err(e)
            }
        }
    }

    /// 取消订单
    #[instrument(skip(self))]
    pub async fn cancel_orders(&self, order_id: &str) -> Result<()> {
        info!("Cancelling order: {}", order_id);
        
        match self.client.cancel_order(order_id).await {
            Ok(response) => {
                if response.success {
                    info!("Order cancelled successfully: {}", response.order_id);
                    Ok(())
                } else {
                    warn!("Failed to cancel order: {}", order_id);
                    anyhow::bail!("Cancel failed")
                }
            }
            Err(e) => {
                error!("Failed to cancel order: {}", e);
                Err(e)
            }
        }
    }

    /// 取消所有订单
    #[instrument(skip(self))]
    pub async fn cancel_all_orders(&self) -> Result<()> {
        info!("Cancelling all orders");
        
        match self.client.cancel_all(None).await {
            Ok(cancelled_ids) => {
                info!("Cancelled {} orders", cancelled_ids.len());
                Ok(())
            }
            Err(e) => {
                error!("Failed to cancel all orders: {}", e);
                Err(e)
            }
        }
    }

}
