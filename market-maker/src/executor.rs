//! 订单执行器 - 使用 poly-client

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use tracing::{info, warn, instrument};

use poly_client::{PolyClient, OrderBook, Side, OrderType};

use crate::config::Config;

#[derive(Clone)]
pub struct Executor {
    client: PolyClient,
    order_size: Decimal,
}

impl Executor {
    pub fn new(config: &Config) -> Self {
        let client = if config.clob.api_key.is_some() && config.clob.api_secret.is_some() {
            // 有认证信息，创建可交易客户端
            PolyClient::with_auth(&config.clob.host, &config.to_auth_config())
        } else {
            // 只读客户端
            PolyClient::new(&config.clob.host)
        };

        Self {
            client,
            order_size: Decimal::from_f64_retain(config.strategy.order_size_usd).unwrap_or(Decimal::from(1000)),
        }
    }

    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        self.client
            .get_orderbook(token_id)
            .await
            .context("Failed to fetch orderbook")
    }

    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn place_orders(&self, token_id: &str, bid_price: f64, ask_price: f64) -> Result<()> {
        let size = self.order_size;
        let bid = Decimal::from_f64_retain(bid_price).context("Invalid bid price")?;
        let ask = Decimal::from_f64_retain(ask_price).context("Invalid ask price")?;

        // 下买单
        match self.client.order.buy(token_id, bid, size).await {
            Ok(resp) => {
                info!("Buy order placed: {}", resp.order_id);
            }
            Err(e) => {
                warn!("Failed to place buy order: {}", e);
            }
        }

        // 下卖单
        match self.client.order.sell(token_id, ask, size).await {
            Ok(resp) => {
                info!("Sell order placed: {}", resp.order_id);
            }
            Err(e) => {
                warn!("Failed to place sell order: {}", e);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cancel_orders(&self, _market_id: &str) -> Result<()> {
        // TODO: 获取并取消该市场的所有订单
        // 需要维护订单 ID 映射
        warn!("Cancel orders not fully implemented");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cancel_all_orders(&self) -> Result<()> {
        match self.client.order.cancel_all().await {
            Ok(cancelled) => {
                info!("Cancelled {} orders", cancelled.len());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_position(&self, token_id: &str) -> Result<Decimal> {
        let positions = self.client.order.get_positions().await?;
        
        Ok(positions
            .iter()
            .find(|p| p.token_id == token_id)
            .map(|p| p.balance)
            .unwrap_or(Decimal::ZERO))
    }
}
