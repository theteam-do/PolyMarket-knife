//! 跟单策略配置

use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

pub use common::{ExecutionConfig, ExecutionMode};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PolygonConfig {
    pub rpc_url: String,
    pub ws_rpc_url: Option<String>,
    #[serde(default)]
    pub private_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClobConfig {
    pub host: String,
    pub ws_market_url: Option<String>,
    pub ws_user_url: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    pub smart_addresses: Vec<String>,
    pub min_trade_size_usd: f64,
    pub max_trade_size_usd: f64,
    pub copy_ratio: f64,
    pub slippage_tolerance: f64,
    pub max_position_per_market: f64,
    pub max_daily_loss: f64,
    pub blacklist: Vec<String>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        if config.polygon.private_key.is_empty() {
            config.polygon.private_key =
                std::env::var("POLYMARKET_PRIVATE_KEY").unwrap_or_default();
        }
        config.execution.enforce_safety()?;
        Ok(config)
    }
}
