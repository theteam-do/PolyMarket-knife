//! 订单执行器

use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use anyhow::{Context, Result};
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::clob::types::{Amount, OrderType, Side};
use polymarket_client_sdk::clob::{Client, Config as SdkConfig};
use polymarket_client_sdk::types::{Decimal, U256};
use polymarket_client_sdk::POLYGON;
use secrecy::ExposeSecret;
use rust_decimal::RoundingStrategy;
use rust_decimal_macros::dec;
use std::str::FromStr;
use tracing::{info, instrument, warn};

use crate::config::Config;
use crate::config::ExecutionMode;
use crate::signal::Signal;

pub struct Executor {
    config: Config,
}

impl Executor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    #[instrument(skip(self), fields(signal = ?signal))]
    pub async fn execute(&self, signal: &Signal) -> Result<Decimal> {
        let position = self.calculate_position(signal.confidence());

        info!(
            "Executing order: symbol={} side={:?} confidence={:.2} position=${}",
            signal.symbol(),
            signal,
            signal.confidence(),
            position
        );

        if self.config.execution.mode == ExecutionMode::Paper {
            return self.simulate_execution(signal, position).await;
        }

        match self.execute_live(signal, position).await {
            Ok(profit) => Ok(profit),
            Err(e) => {
                if self.config.execution.live_failure_fallback_to_paper {
                    warn!("Live execution failed: {}. Falling back to simulation.", e);
                    return self.simulate_execution(signal, position).await;
                }
                anyhow::bail!("live execution failed: {}", e)
            }
        }
    }

    async fn execute_live(&self, signal: &Signal, position: Decimal) -> Result<Decimal> {
        let signer = self.signer()?;
        let market = self
            .config
            .strategy
            .market_for_symbol(signal.symbol())
            .with_context(|| {
                format!(
                    "No symbol_markets mapping configured for {}",
                    signal.symbol()
                )
            })?;
        let token_id = match signal {
            Signal::Buy { .. } => parse_token_id(&market.bullish_token_id)?,
            Signal::Sell { .. } => parse_token_id(&market.bearish_token_id)?,
        };

        let sdk_config = if let Some(proxy_url) = &self.config.clob.proxy_url {
            SdkConfig::builder()
                .use_server_time(true)
                .proxy_url(proxy_url.clone())
                .build()
        } else {
            SdkConfig::builder().use_server_time(true).build()
        };
        let mut auth =
            Client::new(&self.config.clob.host, sdk_config)?.authentication_builder(&signer);
        if let Some(credentials) = self.credentials()? {
            auth = auth.credentials(credentials);
        }
        let client: Client<Authenticated<Normal>> = auth
            .authenticate()
            .await
            .context("Failed to authenticate with CLOB API")?;

        let notional = position.round_dp_with_strategy(2, RoundingStrategy::ToZero);
        let amount = Amount::usdc(notional).context("Failed to build market-buy amount")?;

        let order = client
            .market_order()
            .token_id(token_id)
            .side(Side::Buy)
            .amount(amount)
            .order_type(OrderType::FAK)
            .build()
            .await
            .context("Failed to build order")?;

        let signed_order = client
            .sign(&signer, order)
            .await
            .context("Failed to sign order")?;

        let response = client
            .post_order(signed_order)
            .await
            .context("Failed to submit order")?;

        info!(
            "Volatility order submitted: order_id={} success={} symbol={} notional_usd={}",
            response.order_id,
            response.success,
            signal.symbol(),
            notional
        );
        Ok(Decimal::ZERO)
    }

    async fn simulate_execution(&self, signal: &Signal, position: Decimal) -> Result<Decimal> {
        let base_profit = position * dec!(0.05);
        let confidence_multiplier =
            Decimal::from_f64_retain(signal.confidence()).unwrap_or(dec!(0.5));
        let profit = base_profit * confidence_multiplier * dec!(2.0);

        info!(
            "Simulated execution: position={}, profit={}",
            position, profit
        );
        Ok(profit)
    }

    pub fn estimate_position_usd(&self, signal: &Signal) -> Decimal {
        self.calculate_position(signal.confidence())
    }

    fn calculate_position(&self, confidence: f64) -> Decimal {
        let base = Decimal::from_f64_retain(self.config.strategy.base_position_usd).unwrap();
        let max = Decimal::from_f64_retain(self.config.strategy.max_position_usd).unwrap();

        if confidence >= self.config.strategy.confidence_high {
            max
        } else if confidence >= 0.6 {
            max * dec!(0.3)
        } else {
            base
        }
    }

    fn credentials(&self) -> Result<Option<Credentials>> {
        match (
            self.config
                .clob
                .api_key
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            self.config
                .clob
                .api_secret
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            self.config
                .clob
                .passphrase
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        ) {
            (Some(api_key), Some(api_secret), Some(passphrase)) => Ok(Some(Credentials::new(
                uuid::Uuid::parse_str(api_key).context("invalid clob.api_key UUID")?,
                api_secret.to_string(),
                passphrase.to_string(),
            ))),
            _ => Ok(None),
        }
    }

    fn signer(&self) -> Result<PrivateKeySigner> {
        let private_key = self.config.polygon.private_key.expose_secret().trim();
        if private_key.is_empty() {
            anyhow::bail!("Private key not configured for live execution");
        }
        Ok(PrivateKeySigner::from_str(private_key)?.with_chain_id(Some(POLYGON)))
    }
}

fn parse_token_id(value: &str) -> Result<U256> {
    let raw = value.trim();
    if let Some(hex) = raw.strip_prefix("0x") {
        return U256::from_str_radix(hex, 16).context("invalid hex token id");
    }
    U256::from_str_radix(raw, 10).context("invalid decimal token id")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BinanceConfig, ClobConfig, Config, PolygonConfig, StrategyConfig, SymbolMarketConfig,
    };

    fn create_test_config() -> Config {
        Config {
            polygon: PolygonConfig {
                rpc_url: "https://polygon-rpc.com".to_string(),
                ws_rpc_url: None,
                private_key: "0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7"
                    .to_string()
                    .into(),
            },
            clob: ClobConfig {
                host: "https://clob.polymarket.com".to_string(),
                ws_market_url: None,
                ws_user_url: None,
                api_key: None,
                api_secret: None,
                passphrase: None,
                proxy_url: None,
            },
            binance: BinanceConfig {
                ws_url: "wss://stream.binance.com:9443/ws".to_string(),
                api_key: String::new(),
                api_secret: String::new(),
            },
            strategy: StrategyConfig {
                symbols: vec!["BTCUSDT".to_string()],
                symbol_markets: vec![SymbolMarketConfig {
                    symbol: "BTCUSDT".to_string(),
                    bullish_token_id: "1".to_string(),
                    bearish_token_id: "2".to_string(),
                }],
                volatility_threshold: 0.02,
                momentum_threshold: 0.01,
                base_position_usd: 100.0,
                max_position_usd: 10000.0,
                confidence_high: 0.8,
                max_loss_per_trade: 100.0,
                max_daily_loss: 500.0,
                stop_loss_pct: 0.1,
            },
            execution: common::ExecutionConfig::default(),
        }
    }

    #[test]
    fn test_position_sizing_high_confidence_uses_max() {
        let exec = Executor::new(&create_test_config());
        let position = exec.calculate_position(0.92);
        assert_eq!(position, dec!(10000));
    }

    #[test]
    fn test_position_sizing_mid_confidence_uses_scaled_max() {
        let exec = Executor::new(&create_test_config());
        let position = exec.calculate_position(0.7);
        assert_eq!(position, dec!(3000));
    }

    #[test]
    fn test_position_sizing_low_confidence_uses_base() {
        let exec = Executor::new(&create_test_config());
        let position = exec.calculate_position(0.55);
        assert_eq!(position, dec!(100));
    }

    #[test]
    fn test_parse_token_id_decimal() {
        assert_eq!(parse_token_id("123").unwrap().to_string(), "123");
    }

    #[test]
    fn test_mapping_lookup() {
        let cfg = create_test_config();
        let market = cfg
            .strategy
            .market_for_symbol("btcusdt")
            .expect("mapping exists");
        assert_eq!(market.bearish_token_id, "2");
    }
}
