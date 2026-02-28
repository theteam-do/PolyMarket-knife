//! 波动狩猎策略配置

use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub binance: BinanceConfig,
    pub strategy: StrategyConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PolygonConfig {
    pub rpc_url: String,
    #[serde(default)]
    pub private_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClobConfig {
    pub host: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceConfig {
    pub ws_url: String,
    pub api_key: String,
    #[serde(skip)]
    pub api_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    pub symbols: Vec<String>,
    pub volatility_threshold: f64,
    pub momentum_threshold: f64,
    pub base_position_usd: f64,
    pub max_position_usd: f64,
    pub confidence_high: f64,
    pub max_loss_per_trade: f64,
    pub max_daily_loss: f64,
    pub stop_loss_pct: f64,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        if config.polygon.private_key.is_empty() {
            config.polygon.private_key = std::env::var("POLYMARKET_PRIVATE_KEY").unwrap_or_default();
        }
        Ok(config)
    }
}
