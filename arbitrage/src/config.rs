//! 套利策略配置

use anyhow::Result;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::path::Path;
pub use common::{ClobConfig, ExecutionConfig, ExecutionMode, PolygonConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub strategy: StrategyConfig,
    #[serde(default)]
    pub ctf: CtfConfig,
    #[serde(default)]
    pub quant: QuantConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    pub min_profit_usd: f64,
    pub max_position_per_trade: f64,
    pub gas_price_gwei: u64,
    pub include_all: bool,
    pub exclude_market_ids: Vec<String>,
}

impl StrategyConfig {
    pub fn min_profit(&self) -> Decimal {
        Decimal::from_f64_retain(self.min_profit_usd).unwrap_or(dec!(0.02))
    }

    pub fn max_position(&self) -> Decimal {
        Decimal::from_f64_retain(self.max_position_per_trade).unwrap_or(dec!(1000))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CtfConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub collateral_token: String,
    #[serde(default = "default_collateral_decimals")]
    pub collateral_decimals: u32,
}

impl Default for CtfConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            collateral_token: String::new(),
            collateral_decimals: default_collateral_decimals(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuantConfig {
    #[serde(default)]
    pub fees_bps: f64,
    #[serde(default)]
    pub slippage_bps: f64,
    #[serde(default)]
    pub latency_penalty_bps: f64,
    #[serde(default)]
    pub rebate_bps: f64,
    #[serde(default)]
    pub gas_usd_override: Option<f64>,
    #[serde(default)]
    pub fill_probability_override: Option<f64>,
    #[serde(default)]
    pub implied_prob_override: Option<f64>,
    #[serde(default)]
    pub posterior_prob_override: Option<f64>,
    #[serde(default = "default_net_odds")]
    pub net_odds: f64,
    #[serde(default = "default_fraction_of_kelly")]
    pub fraction_of_kelly: f64,
    #[serde(default)]
    pub bankroll_usd: Option<f64>,
    #[serde(default)]
    pub max_notional_usd: Option<f64>,
    #[serde(default)]
    pub apply_kelly_sizing: bool,
    #[serde(default)]
    pub probability: ProbabilityConfig,
}

impl Default for QuantConfig {
    fn default() -> Self {
        Self {
            fees_bps: 0.0,
            slippage_bps: 0.0,
            latency_penalty_bps: 0.0,
            rebate_bps: 0.0,
            gas_usd_override: None,
            fill_probability_override: None,
            implied_prob_override: None,
            posterior_prob_override: None,
            net_odds: default_net_odds(),
            fraction_of_kelly: default_fraction_of_kelly(),
            bankroll_usd: None,
            max_notional_usd: None,
            apply_kelly_sizing: false,
            probability: ProbabilityConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProbabilityConfig {
    #[serde(default = "default_probability_enabled")]
    pub enabled: bool,
    #[serde(default = "default_probability_prior_prob")]
    pub prior_prob: f64,
    #[serde(default = "default_probability_edge_per_share_scale")]
    pub edge_per_share_scale: f64,
    #[serde(default = "default_probability_gross_edge_scale_usd")]
    pub gross_edge_scale_usd: f64,
}

impl Default for ProbabilityConfig {
    fn default() -> Self {
        Self {
            enabled: default_probability_enabled(),
            prior_prob: default_probability_prior_prob(),
            edge_per_share_scale: default_probability_edge_per_share_scale(),
            gross_edge_scale_usd: default_probability_gross_edge_scale_usd(),
        }
    }
}

impl QuantConfig {
    pub fn validate(&self) -> Result<()> {
        if self.fees_bps < 0.0 {
            anyhow::bail!("quant.fees_bps must be non-negative");
        }
        if self.slippage_bps < 0.0 {
            anyhow::bail!("quant.slippage_bps must be non-negative");
        }
        if self.latency_penalty_bps < 0.0 {
            anyhow::bail!("quant.latency_penalty_bps must be non-negative");
        }
        if self.rebate_bps < 0.0 {
            anyhow::bail!("quant.rebate_bps must be non-negative");
        }
        if self.net_odds <= 0.0 {
            anyhow::bail!("quant.net_odds must be positive");
        }
        if !(0.0 < self.fraction_of_kelly && self.fraction_of_kelly <= 1.0) {
            anyhow::bail!("quant.fraction_of_kelly must be in (0, 1]");
        }
        validate_probability_option(self.implied_prob_override, "quant.implied_prob_override")?;
        validate_probability_option(
            self.posterior_prob_override,
            "quant.posterior_prob_override",
        )?;
        validate_probability_option(
            self.fill_probability_override,
            "quant.fill_probability_override",
        )?;
        self.probability.validate()?;
        validate_non_negative_option(self.gas_usd_override, "quant.gas_usd_override")?;
        validate_positive_option(self.bankroll_usd, "quant.bankroll_usd")?;
        validate_positive_option(self.max_notional_usd, "quant.max_notional_usd")?;
        Ok(())
    }

    pub fn fees_bps_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(self.fees_bps, "quant.fees_bps")
    }

    pub fn slippage_bps_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(self.slippage_bps, "quant.slippage_bps")
    }

    pub fn latency_penalty_bps_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(self.latency_penalty_bps, "quant.latency_penalty_bps")
    }

    pub fn rebate_bps_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(self.rebate_bps, "quant.rebate_bps")
    }

    pub fn net_odds_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(self.net_odds, "quant.net_odds")
    }

    pub fn fraction_of_kelly_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(self.fraction_of_kelly, "quant.fraction_of_kelly")
    }

    pub fn implied_prob_override_decimal(&self) -> Result<Option<Decimal>> {
        self.implied_prob_override
            .map(|value| decimal_from_f64(value, "quant.implied_prob_override"))
            .transpose()
    }

    pub fn posterior_prob_override_decimal(&self) -> Result<Option<Decimal>> {
        self.posterior_prob_override
            .map(|value| decimal_from_f64(value, "quant.posterior_prob_override"))
            .transpose()
    }

    pub fn fill_probability_override_decimal(&self) -> Result<Option<Decimal>> {
        self.fill_probability_override
            .map(|value| decimal_from_f64(value, "quant.fill_probability_override"))
            .transpose()
    }

    pub fn gas_usd_override_decimal(&self) -> Result<Option<Decimal>> {
        self.gas_usd_override
            .map(|value| decimal_from_f64(value, "quant.gas_usd_override"))
            .transpose()
    }

    pub fn bankroll_decimal(&self) -> Result<Option<Decimal>> {
        self.bankroll_usd
            .map(|value| decimal_from_f64(value, "quant.bankroll_usd"))
            .transpose()
    }

    pub fn max_notional_decimal(&self) -> Result<Option<Decimal>> {
        self.max_notional_usd
            .map(|value| decimal_from_f64(value, "quant.max_notional_usd"))
            .transpose()
    }
}

impl ProbabilityConfig {
    pub fn validate(&self) -> Result<()> {
        validate_probability_option(Some(self.prior_prob), "quant.probability.prior_prob")?;
        validate_positive(
            self.edge_per_share_scale,
            "quant.probability.edge_per_share_scale",
        )?;
        validate_positive(
            self.gross_edge_scale_usd,
            "quant.probability.gross_edge_scale_usd",
        )?;
        Ok(())
    }

    pub fn prior_prob_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(self.prior_prob, "quant.probability.prior_prob")
    }

    pub fn edge_per_share_scale_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(
            self.edge_per_share_scale,
            "quant.probability.edge_per_share_scale",
        )
    }

    pub fn gross_edge_scale_usd_decimal(&self) -> Result<Decimal> {
        decimal_from_f64(
            self.gross_edge_scale_usd,
            "quant.probability.gross_edge_scale_usd",
        )
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
        config.ctf.validate()?;
        config.quant.validate()?;
        config.execution.enforce_safety()?;
        Ok(config)
    }
}

fn default_net_odds() -> f64 {
    1.0
}

fn default_fraction_of_kelly() -> f64 {
    0.25
}

fn default_collateral_decimals() -> u32 {
    6
}

fn default_probability_enabled() -> bool {
    true
}

fn default_probability_prior_prob() -> f64 {
    0.50
}

fn default_probability_edge_per_share_scale() -> f64 {
    0.10
}

fn default_probability_gross_edge_scale_usd() -> f64 {
    25.0
}

fn decimal_from_f64(value: f64, name: &str) -> Result<Decimal> {
    Decimal::from_f64(value)
        .ok_or_else(|| anyhow::anyhow!("{name} value {} is not a valid decimal", value))
}

fn validate_probability_option(value: Option<f64>, name: &str) -> Result<()> {
    if let Some(value) = value {
        if !(0.0..=1.0).contains(&value) {
            anyhow::bail!("{name} must be in [0, 1]");
        }
    }
    Ok(())
}

fn validate_positive_option(value: Option<f64>, name: &str) -> Result<()> {
    if let Some(value) = value {
        validate_positive(value, name)?;
    }
    Ok(())
}

fn validate_positive(value: f64, name: &str) -> Result<()> {
    if value <= 0.0 {
        anyhow::bail!("{name} must be positive");
    }
    Ok(())
}

fn validate_non_negative_option(value: Option<f64>, name: &str) -> Result<()> {
    if let Some(value) = value {
        if value < 0.0 {
            anyhow::bail!("{name} must be non-negative");
        }
    }
    Ok(())
}

impl CtfConfig {
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        if self.collateral_token.trim().is_empty() {
            anyhow::bail!("ctf.collateral_token must be set when ctf.enabled = true");
        }
        if self.collateral_decimals == 0 {
            anyhow::bail!("ctf.collateral_decimals must be > 0 when ctf.enabled = true");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn config_parses_optional_quant_runtime_settings() {
        let config: Config = toml::from_str(
            r#"
            [polygon]
            rpc_url = "wss://polygon.example"
            private_key = ""

            [clob]
            host = "https://clob.polymarket.com"

            [strategy]
            min_profit_usd = 0.02
            max_position_per_trade = 1000
            gas_price_gwei = 50
            include_all = true
            exclude_market_ids = []

            [ctf]
            enabled = true
            collateral_token = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"
            collateral_decimals = 6

            [execution]
            mode = "paper"
            environment = "testnet"
            require_explicit_live_ack = true
            live_acknowledged = false
            live_failure_fallback_to_paper = false

            [quant]
            fees_bps = 100
            slippage_bps = 50
            latency_penalty_bps = 25
            rebate_bps = 10
            gas_usd_override = 2.0
            fill_probability_override = 0.8
            posterior_prob_override = 0.70
            net_odds = 1.0
            fraction_of_kelly = 0.50
            bankroll_usd = 1000
            max_notional_usd = 40
            apply_kelly_sizing = true

            [quant.probability]
            enabled = true
            prior_prob = 0.52
            edge_per_share_scale = 0.15
            gross_edge_scale_usd = 30
            "#,
        )
        .expect("config should deserialize");

        assert!(config.ctf.enabled);
        assert_eq!(
            config.ctf.collateral_token,
            "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"
        );
        assert_eq!(config.ctf.collateral_decimals, 6);
        assert_eq!(config.quant.fees_bps, 100.0);
        assert_eq!(config.quant.slippage_bps, 50.0);
        assert_eq!(config.quant.latency_penalty_bps, 25.0);
        assert_eq!(config.quant.rebate_bps, 10.0);
        assert_eq!(config.quant.gas_usd_override, Some(2.0));
        assert_eq!(config.quant.fill_probability_override, Some(0.8));
        assert_eq!(config.quant.posterior_prob_override, Some(0.70));
        assert_eq!(config.quant.net_odds, 1.0);
        assert_eq!(config.quant.fraction_of_kelly, 0.50);
        assert_eq!(config.quant.bankroll_usd, Some(1000.0));
        assert_eq!(config.quant.max_notional_usd, Some(40.0));
        assert!(config.quant.apply_kelly_sizing);
        assert!(config.quant.probability.enabled);
        assert_eq!(config.quant.probability.prior_prob, 0.52);
        assert_eq!(config.quant.probability.edge_per_share_scale, 0.15);
        assert_eq!(config.quant.probability.gross_edge_scale_usd, 30.0);
    }
}
