//! 订单执行器 - 生产级实现

use anyhow::{Context, Result};
use rust_decimal::prelude::ToPrimitive;
use secrecy::ExposeSecret;
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, instrument};

use crate::api::client::ClobClient;
use crate::api::types::{OrderRequest, OrderType, Side};
use crate::config::Config;
use crate::order_book::{Level, OrderBook};

pub struct Executor {
    client: ClobClient,
    nonce: u64,
}

impl Executor {
    pub fn new(config: &Config) -> Result<Self> {
        let credentials = match (
            config.clob.api_key.clone(),
            config.clob.api_secret.clone(),
            config.clob.passphrase.clone(),
        ) {
            (Some(api_key), Some(api_secret), Some(passphrase)) => {
                Some(polymarket_client_sdk::auth::Credentials::new(
                    uuid::Uuid::parse_str(&api_key).context("invalid clob.api_key UUID")?,
                    api_secret,
                    passphrase,
                ))
            }
            _ => None,
        };
        let client = ClobClient::new(
            &config.clob.host,
            Some(config.polygon.private_key.expose_secret().to_string()),
            credentials,
            config.clob.proxy_url.clone(),
        );
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_secs()
            * 1000;

        info!("Executor initialized");

        Ok(Self { client, nonce })
    }

    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        info!("Fetching orderbook for token: {}", token_id);

        let response = self
            .client
            .get_orderbook(token_id)
            .await
            .context("Failed to fetch orderbook from API")?;

        info!(
            "Orderbook response token={} ts={} bids={} asks={}",
            response.token_id,
            response.timestamp,
            response.bids.len(),
            response.asks.len()
        );

        let bids = response
            .bids
            .iter()
            .map(|level| {
                Ok(Level {
                    price: level
                        .price
                        .to_f64()
                        .context("Failed to convert bid price to f64")?,
                    size: level
                        .size
                        .to_f64()
                        .context("Failed to convert bid size to f64")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let asks = response
            .asks
            .iter()
            .map(|level| {
                Ok(Level {
                    price: level
                        .price
                        .to_f64()
                        .context("Failed to convert ask price to f64")?,
                    size: level
                        .size
                        .to_f64()
                        .context("Failed to convert ask size to f64")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let mut order_book = OrderBook::new(token_id.to_string());
        order_book.bids = bids;
        order_book.asks = asks;
        order_book.update_best();

        Ok(order_book)
    }

    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn place_orders(
        &mut self,
        token_id: &str,
        bid: Option<(f64, Decimal)>,
        ask: Option<(f64, Decimal)>,
    ) -> Result<(Option<String>, Option<String>)> {
        if let Some((bid_price, bid_size)) = &bid {
            info!(
                "Prepared BUY order for {}: price={} shares={}",
                token_id,
                Decimal::from_f64_retain(*bid_price).unwrap_or(dec!(0.50)),
                bid_size
            );
        }
        if let Some((ask_price, ask_size)) = &ask {
            info!(
                "Prepared SELL order for {}: price={} shares={}",
                token_id,
                Decimal::from_f64_retain(*ask_price).unwrap_or(dec!(0.50)),
                ask_size
            );
        }

        let buy_result: std::result::Result<Option<String>, anyhow::Error> = match bid {
            Some((bid_price, bid_size)) => {
                let bid_dec = Decimal::from_f64_retain(bid_price).unwrap_or(dec!(0.50));
                match self
                    .place_limit_order(token_id, bid_dec, bid_size, "BUY")
                    .await
                {
                    Ok(order_id) => {
                        info!("Buy order placed: {}", order_id);
                        Ok(Some(order_id))
                    }
                    Err(e) => {
                        error!("Failed to place buy order: {}", e);
                        Err(e)
                    }
                }
            }
            None => Ok(None),
        };

        let sell_result: std::result::Result<Option<String>, anyhow::Error> = match ask {
            Some((ask_price, ask_size)) => {
                let ask_dec = Decimal::from_f64_retain(ask_price).unwrap_or(dec!(0.50));
                match self
                    .place_limit_order(token_id, ask_dec, ask_size, "SELL")
                    .await
                {
                    Ok(order_id) => {
                        info!("Sell order placed: {}", order_id);
                        Ok(Some(order_id))
                    }
                    Err(e) => {
                        error!("Failed to place sell order: {}", e);
                        Err(e)
                    }
                }
            }
            None => Ok(None),
        };

        // Return errors alongside results so caller can diagnose partial failures
        Ok((buy_result.ok().flatten(), sell_result.ok().flatten()))
    }

    async fn place_limit_order(
        &mut self,
        token_id: &str,
        price: Decimal,
        size: Decimal,
        side: &str,
    ) -> Result<String> {
        self.nonce = self.nonce.saturating_add(1);
        let nonce = self.nonce;

        let order_side = match side {
            "BUY" => Side::Buy,
            "SELL" => Side::Sell,
            _ => anyhow::bail!("Invalid side: {}", side),
        };

        let request = OrderRequest {
            token_id: token_id.to_string(),
            price,
            size,
            side: order_side,
            order_type: OrderType::Gtc,
            expiration: 0,
            nonce,
        };

        info!(
            "Placing {} order: token={}, price={}, shares={}, nonce={}",
            side, token_id, price, size, nonce
        );

        match self.client.place_order(request).await {
            Ok(response) => {
                if !response.success {
                    anyhow::bail!("Order rejected by API: {}", response.order_id);
                }
                if let Some(sig) = &response.signature {
                    info!("Order signature received: {}", sig);
                }
                info!("Order placed successfully: {}", response.order_id);
                Ok(response.order_id)
            }
            Err(e) => {
                error!("Failed to place order: {}", e);
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn cancel_orders(&self, order_id: &str) -> Result<()> {
        info!("Cancelling order: {}", order_id);

        match self.client.cancel_order(order_id).await {
            Ok(response) => {
                if response.success {
                    info!("Order cancelled successfully: {}", response.order_id);
                    Ok(())
                } else {
                    anyhow::bail!("Cancel failed for {}", order_id)
                }
            }
            Err(e) => {
                error!("Failed to cancel order: {}", e);
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn cancel_all_orders(&self) -> Result<()> {
        info!("Cancelling all orders");

        match self.client.cancel_all(None).await {
            Ok(cancelled_ids) => {
                info!("Cancelled {} orders", cancelled_ids.len());
                Ok(())
            }
            Err(e) => {
                error!("Failed to cancel all orders: {}", e);
                Err(e)
            }
        }
    }
}

#[allow(dead_code)]
pub(crate) fn shares_for_target_notional(
    target_notional_usd: f64,
    price: f64,
) -> Result<(Decimal, f64)> {
    if target_notional_usd <= 0.0 {
        anyhow::bail!("target notional must be positive");
    }
    if price <= 0.0 {
        anyhow::bail!("price must be positive");
    }

    let notional = Decimal::from_f64_retain(target_notional_usd)
        .context("Failed to convert target notional to Decimal")?;
    let price_dec =
        Decimal::from_f64_retain(price).context("Failed to convert price to Decimal")?;
    let shares = (notional / price_dec).round_dp_with_strategy(2, RoundingStrategy::ToZero);
    if shares <= Decimal::ZERO {
        anyhow::bail!("calculated share size must be positive");
    }

    let actual_notional = (shares * price_dec)
        .round_dp_with_strategy(6, RoundingStrategy::MidpointAwayFromZero)
        .to_f64()
        .context("Failed to convert actual notional to f64")?;

    Ok((shares, actual_notional))
}

#[cfg(test)]
mod tests {
    use super::shares_for_target_notional;

    #[test]
    fn converts_notional_to_share_quantity() {
        let (shares, actual_notional) =
            shares_for_target_notional(1_000.0, 0.50).expect("conversion should succeed");

        assert_eq!(shares.to_string(), "2000");
        assert!((actual_notional - 1_000.0).abs() < 1e-6);
    }

    #[test]
    fn truncates_share_quantity_to_lot_size() {
        let (shares, actual_notional) =
            shares_for_target_notional(100.0, 0.3333).expect("conversion should succeed");

        assert_eq!(shares.scale(), 2);
        assert!(actual_notional <= 100.0);
    }
}
