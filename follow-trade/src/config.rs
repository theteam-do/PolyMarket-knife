//! 跟单策略配置

use anyhow::Result;
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::path::Path;

pub use common::{ClobConfig, ExecutionConfig, ExecutionMode, PolygonConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
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
        if config.polygon.private_key.expose_secret().is_empty() {
            config.polygon.private_key = std::env::var("POLYMARKET_PRIVATE_KEY")
                .unwrap_or_default()
                .into();
        }
        if config.clob.passphrase.is_none() {
            config.clob.passphrase = std::env::var("CLOB_PASSPHRASE").ok();
        }
        if config.clob.proxy_url.is_none() {
            config.clob.proxy_url = std::env::var("CLOB_PROXY_URL").ok();
        }
        config.execution.enforce_safety()?;
        Ok(config)
    }
}
