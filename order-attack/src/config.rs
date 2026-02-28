use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
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
