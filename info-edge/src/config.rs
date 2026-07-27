use anyhow::Result;
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::path::Path;

use common::{ClobConfig, PolygonConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub sources: SourcesConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourcesConfig {
    pub news_apis: Vec<NewsApiConfig>,
    pub keywords: Vec<String>,
    pub gov_websites: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewsApiConfig {
    pub name: String,
    pub url: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    pub confidence_threshold: f64,
    pub max_position_usd: f64,
    pub min_expected_return: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RiskConfig {
    pub max_daily_loss: f64,
    pub legal_review_required: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        if config.polygon.private_key.expose_secret().is_empty() {
            config.polygon.private_key = std::env::var("POLYMARKET_PRIVATE_KEY")
                .unwrap_or_default()
                .trim()
                .to_string()
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
        Ok(config)
    }
}
