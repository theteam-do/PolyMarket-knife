//! 市场扫描器 - 简化版本

use anyhow::Result;
use rust_decimal::Decimal;
use tracing::warn;

use crate::config::Config;

pub struct Scanner {
    #[allow(dead_code)]
    config: Config,
}

impl Scanner {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn scan(&self) -> Result<Vec<MarketPrice>> {
        // TODO: 从 Gamma API 获取市场数据
        warn!("Scanner.scan() - using mock data");

        Ok(vec![MarketPrice {
            market_id: "mock_market".to_string(),
            token_id_yes: "yes_token".to_string(),
            token_id_no: "no_token".to_string(),
            yes_price: Decimal::from_f64_retain(0.45).unwrap(),
            no_price: Decimal::from_f64_retain(0.48).unwrap(),
            volume_24h: Decimal::from_f64_retain(10000.0).unwrap(),
        }])
    }
}

#[derive(Debug, Clone)]
pub struct MarketPrice {
    pub market_id: String,
    pub token_id_yes: String,
    pub token_id_no: String,
    pub yes_price: Decimal,
    pub no_price: Decimal,
    pub volume_24h: Decimal,
}
