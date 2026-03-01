use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub monitor: MonitorConfig,
    pub warning: WarningConfig,
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
    pub ws_market_url: Option<String>,
    pub ws_user_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    /// Gamma 市场列表 API
    pub gamma_markets_url: String,
    /// CLOB 订单簿路径
    pub orderbook_path: String,
    /// HTTP 请求超时（毫秒）
    pub http_timeout_ms: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            gamma_markets_url: "https://gamma-api.polymarket.com/markets".to_string(),
            orderbook_path: "/book".to_string(),
            http_timeout_ms: 10_000,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    /// 攻击 Gas 限制 (故意设置低)
    pub attack_gas_limit: u64,

    /// 是否使用 Nonce 间隙攻击
    pub attack_nonce_gap: bool,

    /// 垄断后目标价差 (5000 = 50%)
    pub target_spread_bps: u32,

    /// 最小流动性目标 (美元)
    pub min_liquidity_usd: f64,

    /// 排除地址
    pub exclude_addresses: Vec<String>,

    /// 每日最大攻击次数
    pub max_attacks_per_day: u32,

    /// 攻击间隔 (秒)
    pub cooldown_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonitorConfig {
    /// 轮询清空超时（秒）
    pub clearing_timeout_seconds: u64,
    /// 轮询间隔（毫秒）
    pub poll_interval_ms: u64,
    /// 判定“接近清空”的最大档位（每边）
    pub max_levels_per_side: usize,
    /// 判定“接近清空”的最大累计深度（每边）
    pub max_depth_per_side: f64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            clearing_timeout_seconds: 30,
            poll_interval_ms: 500,
            max_levels_per_side: 2,
            max_depth_per_side: 100.0,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WarningConfig {
    /// ⚠️ 必须为 true (仅测试网)
    pub testnet_only: bool,

    /// ⚠️ 设置为 true 表示你理解并接受风险
    pub acknowledged: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
