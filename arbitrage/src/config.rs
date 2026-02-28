use anyhow::Result;
use poly_client::AuthConfig;
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

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_auth_config(&self) -> AuthConfig {
        AuthConfig::from_private_key(&self.polygon.private_key, &self.clob.host)
            .expect("Failed to derive API credentials")
    }
}
