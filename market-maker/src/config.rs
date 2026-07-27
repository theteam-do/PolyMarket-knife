//! 配置模块 - 生产级配置管理

use anyhow::{Context, Result};
use common::{ClobConfig, PolygonConfig};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SideMode {
    #[default]
    TwoSided,
    BuyOnly,
    SellOnly,
}

impl SideMode {
    pub fn allows_buy(self) -> bool {
        matches!(self, Self::TwoSided | Self::BuyOnly)
    }

    pub fn allows_sell(self) -> bool {
        matches!(self, Self::TwoSided | Self::SellOnly)
    }
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
    #[serde(default)]
    pub side_mode: SideMode,
    /// Metrics HTTP server bind address (default: 127.0.0.1:9090)
    #[serde(default = "default_metrics_addr")]
    pub metrics_bind_addr: String,
}

fn default_metrics_addr() -> String {
    "127.0.0.1:9090".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct RiskConfig {
    pub max_position_usd: f64,
    pub max_loss_per_day: f64,
    pub stop_loss_pct: f64,
    pub max_orders: usize,
    pub max_order_size_usd: f64,
    /// Per-market position concentration limit as fraction of max_position_usd (default: 0.3)
    #[serde(default = "default_concentration_pct")]
    pub max_market_concentration_pct: f64,
}

fn default_concentration_pct() -> f64 {
    0.3
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| "Failed to read config file")?;

        let mut config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config file")?;

        if config.polygon.private_key.expose_secret().is_empty() {
            config.polygon.private_key = SecretString::from(
                std::env::var("POLYMARKET_PRIVATE_KEY")
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            );
        }

        config.validate()?;
        Ok(config)
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self {
            polygon: PolygonConfig {
                rpc_url: std::env::var("POLYGON_RPC_URL")
                    .unwrap_or_else(|_| "https://polygon-bor-rpc.publicnode.com".to_string()),
                ws_rpc_url: None,
                private_key: SecretString::from(
                    std::env::var("POLYMARKET_PRIVATE_KEY").unwrap_or_default(),
                ),
            },
            clob: ClobConfig {
                host: std::env::var("CLOB_HOST")
                    .unwrap_or_else(|_| "https://clob.polymarket.com".to_string()),
                ws_market_url: std::env::var("CLOB_WS_MARKET_URL").ok(),
                ws_user_url: std::env::var("CLOB_WS_USER_URL").ok(),
                api_key: std::env::var("CLOB_API_KEY").ok(),
                api_secret: std::env::var("CLOB_API_SECRET").ok(),
                passphrase: std::env::var("CLOB_PASSPHRASE").ok(),
                proxy_url: std::env::var("CLOB_PROXY_URL").ok(),
            },
            strategy: StrategyConfig {
                market_ids: vec![],
                spread_bps: 100,
                order_size_usd: 1000.0,
                refresh_interval_ms: 100,
                skew_inventory: true,
                min_spread_bps: 50,
                max_spread_bps: 500,
                side_mode: SideMode::TwoSided,
                metrics_bind_addr: default_metrics_addr(),
            },
            risk: RiskConfig {
                max_position_usd: 10000.0,
                max_loss_per_day: 500.0,
                stop_loss_pct: 5.0,
                max_orders: 10,
                max_order_size_usd: 5000.0,
                max_market_concentration_pct: default_concentration_pct(),
            },
        })
    }

    fn validate(&self) -> Result<()> {
        if self.polygon.private_key.expose_secret().is_empty() {
            anyhow::bail!(
                "Private key is required. Set POLYMARKET_PRIVATE_KEY environment variable"
            );
        }

        if self.strategy.spread_bps < self.strategy.min_spread_bps {
            anyhow::bail!("spread_bps cannot be less than min_spread_bps");
        }

        if self.strategy.spread_bps > self.strategy.max_spread_bps {
            anyhow::bail!("spread_bps cannot be greater than max_spread_bps");
        }

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
    use secrecy::ExposeSecret;
    use super::{Config, SideMode};
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
rpc_url = "https://polygon-bor-rpc.publicnode.com"
private_key = "0xabc"

[clob]
host = "https://clob.polymarket.com"
proxy_url = "socks5h://127.0.0.1:7890"

[strategy]
market_ids = ["1"]
spread_bps = 100
order_size_usd = 100.0
refresh_interval_ms = 100
skew_inventory = true
min_spread_bps = 50
max_spread_bps = 200
side_mode = "buy_only"

[risk]
max_position_usd = 1000.0
max_loss_per_day = 100.0
stop_loss_pct = 5.0
max_orders = 10
max_order_size_usd = 500.0
"#,
        );

        let cfg = Config::load(&path).expect("config should load");
        assert_eq!(cfg.polygon.private_key.expose_secret(), "0xabc");
        assert_eq!(cfg.strategy.side_mode, SideMode::BuyOnly);
        assert_eq!(cfg.strategy.metrics_bind_addr, "127.0.0.1:9090");
        assert!((cfg.risk.max_market_concentration_pct - 0.3).abs() < 1e-6);

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
rpc_url = "https://polygon-bor-rpc.publicnode.com"

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
        assert_eq!(cfg.polygon.private_key.expose_secret(), "0xenv");
        assert_eq!(cfg.strategy.side_mode, SideMode::TwoSided);

        let _ = fs::remove_file(path);
        restore_env(original_env);
    }
}
