//! 交易执行器 - 使用官方 SDK

use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use anyhow::{Context, Result};
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;
use polymarket_client_sdk::clob::types::{OrderType, Side as SdkSide};
use polymarket_client_sdk::clob::{Client, Config as SdkConfig};
use polymarket_client_sdk::types::{Decimal, U256};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use secrecy::ExposeSecret;
use std::str::FromStr;
use tracing::{info, instrument};

use crate::config::Config;
use crate::nlp::Direction;
use crate::signal::Signal;

pub struct Executor {
    client: Client<Authenticated<Normal>>,
    signer: PrivateKeySigner,
    config: Config,
}

impl Executor {
    pub async fn new(config: &Config) -> Result<Self> {
        let private_key = if config.polygon.private_key.expose_secret().is_empty() {
            std::env::var(PRIVATE_KEY_VAR)
                .context("Need POLYMARKET_PRIVATE_KEY environment variable")?
        } else {
            config.polygon.private_key.expose_secret().to_string()
        };

        let signer = PrivateKeySigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

        let sdk_config = if let Some(proxy_url) = &config.clob.proxy_url {
            SdkConfig::builder()
                .use_server_time(true)
                .proxy_url(proxy_url.clone())
                .build()
        } else {
            SdkConfig::builder().use_server_time(true).build()
        };
        let mut auth = Client::new(&config.clob.host, sdk_config)?.authentication_builder(&signer);
        if let Some(credentials) = credentials_from_config(config)? {
            auth = auth.credentials(credentials);
        }
        let client = auth
            .authenticate()
            .await
            .context("Failed to authenticate")?;

        Ok(Self {
            client,
            signer,
            config: config.clone(),
        })
    }

    #[instrument(skip(self), fields(signal = ?signal))]
    pub async fn execute(&self, signal: &Signal) -> Result<()> {
        let token_id = parse_market_token_id(&signal.market)?;
        let price = self.reference_price(token_id).await?;
        let size = self.calculate_size(signal.confidence, price)?;

        info!(
            "Executing order: direction={:?} token_id={} confidence={:.2} price={} size={}",
            signal.direction, token_id, signal.confidence, price, size
        );

        let side = match signal.direction {
            Direction::Yes => SdkSide::Buy,
            Direction::No => SdkSide::Sell,
            Direction::Neutral => return Ok(()),
        };

        let order = self
            .client
            .limit_order()
            .token_id(token_id)
            .order_type(OrderType::GTC)
            .price(price)
            .size(size)
            .side(side)
            .build()
            .await?;

        let signed_order = self.client.sign(&self.signer, order).await?;
        let resp = self.client.post_order(signed_order).await?;

        info!(
            "Order placed: order_id={} success={}",
            resp.order_id, resp.success
        );

        Ok(())
    }

    fn calculate_size(&self, confidence: f64, price: Decimal) -> Result<Decimal> {
        if price <= Decimal::ZERO {
            anyhow::bail!("reference price must be positive");
        }

        let confidence = confidence.clamp(0.0, 1.0);
        let min_confidence = self.config.strategy.confidence_threshold.clamp(0.0, 1.0);
        let scaled_confidence = confidence.max(min_confidence);
        let notional =
            Decimal::from_f64_retain(self.config.strategy.max_position_usd * scaled_confidence)
                .context("failed to convert max_position_usd to Decimal")?;
        let size = (notional / price).round_dp(2);

        if size <= Decimal::ZERO {
            anyhow::bail!("calculated size must be positive");
        }

        Ok(size)
    }

    async fn reference_price(&self, token_id: U256) -> Result<Decimal> {
        let request = OrderBookSummaryRequest::builder()
            .token_id(token_id)
            .build();
        let book = self
            .client
            .order_book(&request)
            .await
            .context("Failed to fetch orderbook for signal")?;

        match (book.bids.first(), book.asks.first()) {
            (Some(bid), Some(ask)) => Ok((bid.price + ask.price) / Decimal::from(2u32)),
            (Some(bid), None) => Ok(bid.price),
            (None, Some(ask)) => Ok(ask.price),
            (None, None) => anyhow::bail!("orderbook is empty for token {}", token_id),
        }
    }
}

fn credentials_from_config(config: &Config) -> Result<Option<Credentials>> {
    match (
        config
            .clob
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty()),
        config
            .clob
            .api_secret
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty()),
        config
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

fn parse_market_token_id(market: &str) -> Result<U256> {
    let raw = market.trim();
    if let Some(hex) = raw.strip_prefix("0x") {
        return U256::from_str_radix(hex, 16)
            .context("market must be a valid 0x-prefixed token id");
    }

    if raw.chars().all(|ch| ch.is_ascii_digit()) {
        return U256::from_str_radix(raw, 10).context("market must be a valid decimal token id");
    }

    anyhow::bail!("market must be a decimal or 0x-prefixed token id")
}

#[cfg(test)]
mod tests {
    use super::parse_market_token_id;

    #[test]
    fn parses_hex_token_id() {
        let token = parse_market_token_id(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .expect("hex token id should parse");
        assert_eq!(token.to_string(), "1");
    }

    #[test]
    fn rejects_placeholder_market_name() {
        assert!(parse_market_token_id("general_market").is_err());
    }
}
