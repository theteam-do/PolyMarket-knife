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
    #[serde(skip)]
    pub private_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClobConfig {
    pub host: String,
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
    /// 监控的交易对
    pub symbols: Vec<String>,

    /// 波动率阈值 (0.02 = 2%)
    pub volatility_threshold: f64,

    /// 动量阈值 (0.01 = 1%)
    pub momentum_threshold: f64,

    /// 基础仓位 (美元) - 用于低置信度信号
    pub base_position_usd: f64,

    /// 最大仓位 (美元) - 用于高置信度信号
    pub max_position_usd: f64,

    /// 高置信度阈值
    pub confidence_high: f64,

    /// 单笔最大亏损 (美元)
    pub max_loss_per_trade: f64,

    /// 日最大亏损 (美元)
    pub max_daily_loss: f64,

    /// 止损百分比
    pub stop_loss_pct: f64,
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
