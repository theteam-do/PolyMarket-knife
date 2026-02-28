//! 目标扫描器

use anyhow::Result;
use reqwest::Client;
use tracing::{instrument};

use crate::config::StrategyConfig;

pub struct TargetScanner {
    client: Client,
    config: StrategyConfig,
}

impl TargetScanner {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
            config: config.clone(),
        }
    }

    #[instrument(skip(self))]
    pub async fn scan(&self) -> Result<Vec<TargetMarket>> {
        // TODO: 扫描高流动性市场
        // 从 Gamma API 获取活跃市场
        // 排除配置的地址
        
        // 模拟返回
        Ok(vec![])
    }
}

#[derive(Debug)]
pub struct TargetMarket {
    pub market: String,
    pub liquidity_usd: f64,
    pub market_makers: Vec<String>,
}
