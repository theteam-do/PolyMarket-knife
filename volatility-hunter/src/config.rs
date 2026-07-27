//! 波动狩猎策略配置

use anyhow::Result;
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::path::Path;

pub use common::{ClobConfig, ExecutionConfig, ExecutionMode, PolygonConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub binance: BinanceConfig,
    pub strategy: StrategyConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceConfig {
    pub ws_url: String,
    pub api_key: String,
    #[serde(skip)]
    pub api_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SymbolMarketConfig {
    pub symbol: String,
    pub bullish_token_id: String,
    pub bearish_token_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    pub symbols: Vec<String>,
    pub symbol_markets: Vec<SymbolMarketConfig>,
    pub volatility_threshold: f64,
    pub momentum_threshold: f64,
    pub base_position_usd: f64,
    pub max_position_usd: f64,
    pub confidence_high: f64,
    pub max_loss_per_trade: f64,
    pub max_daily_loss: f64,
    pub stop_loss_pct: f64,
}

impl StrategyConfig {
    pub fn market_for_symbol(&self, symbol: &str) -> Option<&SymbolMarketConfig> {
        self.symbol_markets
            .iter()
            .find(|market| market.symbol.eq_ignore_ascii_case(symbol))
    }
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
        if config.clob.api_key.is_none() {
            config.clob.api_key = std::env::var("CLOB_API_KEY").ok();
        }
        if config.clob.api_secret.is_none() {
            config.clob.api_secret = std::env::var("CLOB_API_SECRET").ok();
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
