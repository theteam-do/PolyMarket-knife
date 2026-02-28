//! 目标扫描器

use crate::config::StrategyConfig;
use anyhow::Result;

pub struct TargetScanner {
    #[allow(dead_code)]
    config: StrategyConfig,
}

impl TargetScanner {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn scan(&self) -> Result<Vec<TargetMarket>> {
        Ok(vec![])
    }
}

#[derive(Debug)]
pub struct TargetMarket {
    pub market: String,
    // pub liquidity_usd: f64,
    // pub market_makers: Vec<String>,
}
