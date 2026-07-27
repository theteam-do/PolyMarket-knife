//! 交易复制器 - 使用官方 SDK 复制聪明钱交易

use anyhow::{Context, Result};
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use std::str::FromStr;
use tracing::{info, instrument, warn};

use secrecy::ExposeSecret;
use alloy::primitives::ChainId;
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::clob::types::{OrderType as SdkOrderType, Side as SdkSide};
use polymarket_client_sdk::clob::{Client as ClobSdkClient, Config as ClobConfig};
use polymarket_client_sdk::types::U256;

use crate::config::Config;
use crate::config::ExecutionMode;
use crate::monitor::TradeEvent;

const POLYGON_CHAIN_ID: ChainId = 137;

#[derive(Debug, Clone)]
pub struct CopyOutcome {
    pub copied_notional_usd: Decimal,
    pub share_size: Decimal,
    pub order_id: Option<String>,
    pub realized_pnl: Option<Decimal>,
    pub simulated: bool,
}

pub struct TradeCopier {
    config: Config,
}

impl TradeCopier {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    #[instrument(skip(self), fields(trade = ?trade))]
    pub async fn copy(&self, trade: &TradeEvent) -> Result<CopyOutcome> {
        let target_notional_usd = self.calculate_copy_notional(trade.size_usd);
        let (share_size, actual_notional_usd) =
            self.share_size_for_notional(target_notional_usd, trade.price)?;

        info!(
            "Copying trade: market={} side={:?} original_notional=${} copied_notional=${} price={} share_size={}",
            trade.market,
            trade.side,
            trade.size_usd,
            actual_notional_usd,
            trade.price,
            share_size
        );

        if self.config.execution.mode == ExecutionMode::Paper {
            return Ok(self
                .simulate_execution(actual_notional_usd, share_size)
                .await);
        }

        match self.execute_live(trade, share_size).await {
            Ok(order_id) => {
                info!("Order placed successfully: {}", order_id);
                Ok(CopyOutcome {
                    copied_notional_usd: actual_notional_usd,
                    share_size,
                    order_id: Some(order_id),
                    realized_pnl: None,
                    simulated: false,
                })
            }
            Err(e) => {
                if self.config.execution.live_failure_fallback_to_paper {
                    warn!("Live execution failed: {}. Falling back to simulation.", e);
                    return Ok(self
                        .simulate_execution(actual_notional_usd, share_size)
                        .await);
                }
                anyhow::bail!("live copy execution failed: {}", e);
            }
        }
    }

    async fn execute_live(&self, trade: &TradeEvent, share_size: Decimal) -> Result<String> {
        let private_key = self.config.polygon.private_key.expose_secret().trim();
        if private_key.is_empty() {
            anyhow::bail!("Private key not configured for live execution");
        }

        let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
        let signer = LocalSigner::from_str(pk)
            .context("Failed to parse private key")?
            .with_chain_id(Some(POLYGON_CHAIN_ID));

        let host = self.config.clob.host.trim_end_matches('/');
        let sdk_config = if let Some(proxy_url) = &self.config.clob.proxy_url {
            ClobConfig::builder().proxy_url(proxy_url.clone()).build()
        } else {
            ClobConfig::default()
        };
        let mut auth = ClobSdkClient::new(host, sdk_config)?.authentication_builder(&signer);
        if let Some(credentials) = self.credentials()? {
            auth = auth.credentials(credentials);
        }
        let sdk_client = auth
            .authenticate()
            .await
            .context("Failed to authenticate with CLOB API")?;

        let token_id_str = trade
            .market_id
            .strip_prefix("0x")
            .unwrap_or(&trade.market_id);
        let token_id = U256::from_str_radix(token_id_str, 16)
            .context("Failed to parse token_id from trade event")?;

        let side = match trade.side {
            crate::monitor::Side::Buy => SdkSide::Buy,
            crate::monitor::Side::Sell => SdkSide::Sell,
        };

        let tick_size = sdk_client
            .tick_size(token_id)
            .await
            .context("Failed to fetch tick size")?
            .minimum_tick_size
            .as_decimal();
        let decimals = tick_size.scale();

        let price = trade.price.round_dp(decimals);
        let share_size = share_size.round_dp_with_strategy(2, RoundingStrategy::ToZero);

        let order = sdk_client
            .limit_order()
            .token_id(token_id)
            .side(side)
            .price(price)
            .size(share_size)
            .order_type(SdkOrderType::GTC)
            .build()
            .await
            .context("Failed to build order")?;

        let signed_order = sdk_client
            .sign(&signer, order)
            .await
            .context("Failed to sign order")?;

        let response = sdk_client
            .post_order(signed_order)
            .await
            .context("Failed to submit order")?;

        Ok(response.order_id.to_string())
    }

    async fn simulate_execution(
        &self,
        copied_notional_usd: Decimal,
        share_size: Decimal,
    ) -> CopyOutcome {
        info!(
            "Simulated copy submission: copied_notional=${} share_size={}",
            copied_notional_usd, share_size
        );
        CopyOutcome {
            copied_notional_usd,
            share_size,
            order_id: None,
            realized_pnl: None,
            simulated: true,
        }
    }

    fn calculate_copy_notional(&self, original_notional_usd: Decimal) -> Decimal {
        let copy_ratio =
            Decimal::from_f64_retain(self.config.strategy.copy_ratio).unwrap_or(dec!(1.0));
        let notional = original_notional_usd * copy_ratio;

        let min_notional =
            Decimal::from_f64_retain(self.config.strategy.min_trade_size_usd).unwrap_or(dec!(5.0));
        let max_notional = Decimal::from_f64_retain(self.config.strategy.max_trade_size_usd)
            .unwrap_or(dec!(1000.0));

        notional.clamp(min_notional, max_notional)
    }

    fn credentials(&self) -> Result<Option<Credentials>> {
        match (
            self.config
                .clob
                .api_key
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
            self.config
                .clob
                .api_secret
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
            self.config
                .clob
                .passphrase
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
        ) {
            (Some(api_key), Some(api_secret), Some(passphrase)) => Ok(Some(Credentials::new(
                uuid::Uuid::parse_str(api_key).context("invalid clob.api_key UUID")?,
                api_secret.to_string(),
                passphrase.to_string(),
            ))),
            _ => Ok(None),
        }
    }

    fn share_size_for_notional(
        &self,
        target_notional_usd: Decimal,
        price: Decimal,
    ) -> Result<(Decimal, Decimal)> {
        if target_notional_usd <= Decimal::ZERO {
            anyhow::bail!("target notional must be positive");
        }
        if price <= Decimal::ZERO {
            anyhow::bail!("price must be positive");
        }

        let share_size =
            (target_notional_usd / price).round_dp_with_strategy(2, RoundingStrategy::ToZero);
        if share_size <= Decimal::ZERO {
            anyhow::bail!("calculated share size must be positive");
        }

        let actual_notional =
            (share_size * price).round_dp_with_strategy(6, RoundingStrategy::MidpointAwayFromZero);
        Ok((share_size, actual_notional))
    }
}

#[cfg(test)]
mod tests {
    use super::TradeCopier;
    use crate::config::{ClobConfig, Config, ExecutionConfig, PolygonConfig, StrategyConfig};
    use crate::monitor::{Side, TradeEvent};
    use rust_decimal_macros::dec;

    fn config() -> Config {
        Config {
            polygon: PolygonConfig {
                rpc_url: "http://localhost".to_string(),
                ws_rpc_url: None,
                private_key: "0x1".to_string().into(),
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
            strategy: StrategyConfig {
                smart_addresses: vec![],
                min_trade_size_usd: 50.0,
                max_trade_size_usd: 200.0,
                copy_ratio: 0.5,
                slippage_tolerance: 0.02,
                max_position_per_market: 1_000.0,
                max_daily_loss: 100.0,
                blacklist: vec![],
            },
            execution: ExecutionConfig::default(),
        }
    }

    fn trade() -> TradeEvent {
        TradeEvent {
            from: "0xabc".to_string(),
            market: "0x01".to_string(),
            market_id: "0x01".to_string(),
            side: Side::Buy,
            size_usd: dec!(300),
            price: dec!(0.25),
            timestamp: 0,
        }
    }

    #[test]
    fn calculates_copy_notional_in_usd() {
        let copier = TradeCopier::new(&config());
        assert_eq!(copier.calculate_copy_notional(dec!(300)), dec!(150));
        assert_eq!(copier.calculate_copy_notional(dec!(20)), dec!(50));
        assert_eq!(copier.calculate_copy_notional(dec!(1000)), dec!(200));
    }

    #[test]
    fn converts_usd_notional_to_share_quantity() {
        let copier = TradeCopier::new(&config());
        let (shares, notional) = copier
            .share_size_for_notional(dec!(150), dec!(0.25))
            .expect("conversion should succeed");

        assert_eq!(shares, dec!(600));
        assert_eq!(notional, dec!(150));
    }

    #[tokio::test]
    async fn paper_mode_reports_notional_and_shares_separately() {
        let copier = TradeCopier::new(&config());
        let outcome = copier
            .copy(&trade())
            .await
            .expect("paper copy should succeed");

        assert!(outcome.simulated);
        assert_eq!(outcome.copied_notional_usd, dec!(150));
        assert_eq!(outcome.share_size, dec!(600));
        assert!(outcome.realized_pnl.is_none());
    }
}
