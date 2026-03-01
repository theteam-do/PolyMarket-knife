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
    #[serde(default)]
    pub execution: ExecutionConfig,
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

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    Paper,
    Live,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Paper
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeEnvironment {
    Testnet,
    Mainnet,
}

impl Default for RuntimeEnvironment {
    fn default() -> Self {
        Self::Testnet
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutionConfig {
    #[serde(default)]
    pub mode: ExecutionMode,
    #[serde(default)]
    pub environment: RuntimeEnvironment,
    #[serde(default = "default_true")]
    pub require_explicit_live_ack: bool,
    #[serde(default)]
    pub live_acknowledged: bool,
    #[serde(default)]
    pub live_failure_fallback_to_paper: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Paper,
            environment: RuntimeEnvironment::Testnet,
            require_explicit_live_ack: true,
            live_acknowledged: false,
            live_failure_fallback_to_paper: false,
        }
    }
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        if config.polygon.private_key.is_empty() {
            config.polygon.private_key = std::env::var("POLYMARKET_PRIVATE_KEY").unwrap_or_default();
        }
        config.enforce_execution_safety()?;
        Ok(config)
    }

    pub fn enforce_execution_safety(&self) -> Result<()> {
        if self.execution.mode == ExecutionMode::Live
            && self.execution.require_explicit_live_ack
            && !self.execution.live_acknowledged
        {
            anyhow::bail!(
                "live mode requires explicit acknowledgement: set [execution].live_acknowledged = true"
            );
        }

        Ok(())
    }
}
