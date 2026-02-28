//! 套利策略配置

use anyhow::Result;
use polymarket_client_sdk::types::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PolygonConfig {
    pub rpc_url: String,
    #[serde(skip)]
    pub private_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClobConfig {
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    pub min_profit_usd: f64,
    pub max_position_per_trade: f64,
    pub scan_interval_ms: u64,
    pub gas_price_gwei: u64,
    pub include_all: bool,
    pub exclude_market_ids: Vec<String>,
}

impl StrategyConfig {
    pub fn min_profit(&self) -> Decimal {
        Decimal::from_f64_retain(self.min_profit_usd).unwrap_or(dec!(0.02))
    }

    pub fn max_position(&self) -> Decimal {
        Decimal::from_f64_retain(self.max_position_per_trade).unwrap_or(dec!(1000))
    }

    pub fn scan_interval_ms(&self) -> u64 {
        self.scan_interval_ms
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
