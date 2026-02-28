use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
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
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    pub market_ids: Vec<String>,
    pub spread_bps: u32,
    pub order_size_usd: f64,
    pub refresh_interval_ms: u64,
    pub skew_inventory: bool,
    pub min_spread_bps: u32,
    pub max_spread_bps: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RiskConfig {
    pub max_position_usd: f64,
    pub max_loss_per_day: f64,
    pub stop_loss_pct: f64,
    pub max_orders: usize,
    pub max_order_size_usd: f64,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
