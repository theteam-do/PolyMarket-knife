//! 配置模块 - 生产级配置管理

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// 主配置
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
}

/// Polygon 配置
#[derive(Debug, Deserialize, Clone)]
pub struct PolygonConfig {
    pub rpc_url: String,
    #[serde(default)]
    pub private_key: String,
}

/// CLOB 配置
#[derive(Debug, Deserialize, Clone)]
pub struct ClobConfig {
    pub host: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}

/// 策略配置
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

/// 风控配置
#[derive(Debug, Deserialize, Clone)]
pub struct RiskConfig {
    pub max_position_usd: f64,
    pub max_loss_per_day: f64,
    pub stop_loss_pct: f64,
    pub max_orders: usize,
    pub max_order_size_usd: f64,
}

impl Config {
    /// 从文件加载配置
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| "Failed to read config file")?;
        
        let mut config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;

        if config.polygon.private_key.is_empty() {
            config.polygon.private_key = std::env::var("POLYMARKET_PRIVATE_KEY").unwrap_or_default();
        }
        
        // 验证配置
        config.validate()?;
        
        Ok(config)
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            polygon: PolygonConfig {
                rpc_url: std::env::var("POLYGON_RPC_URL")
                    .unwrap_or_else(|_| "https://polygon-rpc.com".to_string()),
                private_key: std::env::var("POLYMARKET_PRIVATE_KEY")
                    .unwrap_or_default(),
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

    /// 验证配置
    fn validate(&self) -> Result<()> {
        // 验证私钥
        if self.polygon.private_key.is_empty() {
            anyhow::bail!("Private key is required. Set POLYMARKET_PRIVATE_KEY environment variable");
        }

        // 验证价差
        if self.strategy.spread_bps < self.strategy.min_spread_bps {
            anyhow::bail!("spread_bps cannot be less than min_spread_bps");
        }

        if self.strategy.spread_bps > self.strategy.max_spread_bps {
            anyhow::bail!("spread_bps cannot be greater than max_spread_bps");
        }

        // 验证风控
        if self.risk.max_loss_per_day <= 0.0 {
            anyhow::bail!("max_loss_per_day must be positive");
        }

        if self.risk.max_position_usd <= 0.0 {
            anyhow::bail!("max_position_usd must be positive");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::fs;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn restore_env(original: Option<String>) {
        if let Some(value) = original {
            std::env::set_var("POLYMARKET_PRIVATE_KEY", value);
        } else {
            std::env::remove_var("POLYMARKET_PRIVATE_KEY");
        }
    }

    fn write_temp_config(contents: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mm-config-{}.toml", nanos));
        fs::write(&path, contents).expect("failed to write temp config");
        path.to_string_lossy().to_string()
    }

    #[test]
    fn load_uses_private_key_from_file() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let original_env = std::env::var("POLYMARKET_PRIVATE_KEY").ok();
        std::env::remove_var("POLYMARKET_PRIVATE_KEY");

        let path = write_temp_config(
            r#"
[polygon]
rpc_url = "https://polygon-rpc.com"
private_key = "0xabc"

[clob]
host = "https://clob.polymarket.com"

[strategy]
market_ids = ["1"]
spread_bps = 100
order_size_usd = 100.0
refresh_interval_ms = 100
skew_inventory = true
min_spread_bps = 50
max_spread_bps = 200

[risk]
max_position_usd = 1000.0
max_loss_per_day = 100.0
stop_loss_pct = 5.0
max_orders = 10
max_order_size_usd = 500.0
"#,
        );

        let cfg = Config::load(&path).expect("config should load");
        assert_eq!(cfg.polygon.private_key, "0xabc");

        let _ = fs::remove_file(path);
        restore_env(original_env);
    }

    #[test]
    fn load_falls_back_to_env_private_key() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let original_env = std::env::var("POLYMARKET_PRIVATE_KEY").ok();
        std::env::set_var("POLYMARKET_PRIVATE_KEY", "0xenv");

        let path = write_temp_config(
            r#"
[polygon]
rpc_url = "https://polygon-rpc.com"

[clob]
host = "https://clob.polymarket.com"

[strategy]
market_ids = ["1"]
spread_bps = 100
order_size_usd = 100.0
refresh_interval_ms = 100
skew_inventory = true
min_spread_bps = 50
max_spread_bps = 200

[risk]
max_position_usd = 1000.0
max_loss_per_day = 100.0
stop_loss_pct = 5.0
max_orders = 10
max_order_size_usd = 500.0
"#,
        );

        let cfg = Config::load(&path).expect("config should load");
        assert_eq!(cfg.polygon.private_key, "0xenv");

        let _ = fs::remove_file(path);
        restore_env(original_env);
    }
}
