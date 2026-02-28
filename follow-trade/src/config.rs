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
    /// 监控的聪明钱地址列表
    pub smart_addresses: Vec<String>,

    /// 最小跟单金额 (美元)
    pub min_trade_size_usd: f64,

    /// 最大跟单金额 (美元)
    pub max_trade_size_usd: f64,

    /// 跟单比例 (0.1 = 10%)
    pub copy_ratio: f64,

    /// 滑点容忍度 (0.02 = 2%)
    pub slippage_tolerance: f64,

    /// 单市场最大持仓 (美元)
    pub max_position_per_market: f64,

    /// 日最大亏损 (美元)
    pub max_daily_loss: f64,

    /// 黑名单市场
    pub blacklist: Vec<String>,
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
