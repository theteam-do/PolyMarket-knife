use anyhow::Result;
use ethers::signers::LocalWallet;
use poly_client::AuthConfig;
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

    pub fn from_env() -> Result<Self> {
        Ok(Self {
            polygon: PolygonConfig {
                rpc_url: std::env::var("POLYGON_RPC_URL")
                    .unwrap_or_else(|_| "https://polygon-rpc.com".to_string()),
                private_key: std::env::var("PRIVATE_KEY").unwrap_or_default(),
            },
            clob: ClobConfig {
                host: std::env::var("CLOB_HOST")
                    .unwrap_or_else(|_| "https://clob.polymarket.com".to_string()),
                api_key: std::env::var("CLOB_API_KEY").ok(),
                api_secret: std::env::var("CLOB_API_SECRET").ok(),
            },
            strategy: StrategyConfig {
                market_ids: vec![],
                spread_bps: 100,
                order_size_usd: 1000.0,
                refresh_interval_ms: 100,
                skew_inventory: true,
                min_spread_bps: 50,
                max_spread_bps: 500,
            },
            risk: RiskConfig {
                max_position_usd: 10000.0,
                max_loss_per_day: 500.0,
                stop_loss_pct: 5.0,
                max_orders: 10,
                max_order_size_usd: 5000.0,
            },
        })
    }

    pub fn to_auth_config(&self) -> AuthConfig {
        AuthConfig::from_private_key(&self.polygon.private_key, &self.clob.host)
            .expect("Failed to derive API credentials")
    }
}
