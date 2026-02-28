//! 链上交易监控器

use anyhow::Result;
use rust_decimal::Decimal;
use tracing::warn;

use crate::config::Config;

pub struct ChainMonitor {
    #[allow(dead_code)]
    config: Config,
}

impl ChainMonitor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn fetch_trades(&self) -> Result<Vec<TradeEvent>> {
        // TODO: 从 Data API 获取聪明钱交易
        warn!("fetch_trades() - using mock data");

        Ok(vec![TradeEvent {
            from: "0xSmartMoney".to_string(),
            market: "mock_market".to_string(),
            market_id: "mock_id".to_string(),
            side: Side::Buy,
            size_usd: 1000.0,
            price: 0.50,
            timestamp: 0,
        }])
    }
}

#[derive(Debug, Clone)]
pub struct TradeEvent {
    pub from: String,
    pub market: String,
    pub market_id: String,
    pub side: Side,
    pub size_usd: f64,
    pub price: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}
