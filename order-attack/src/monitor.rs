//! 订单簿监控器

use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

pub struct OrderbookMonitor {
    config: Config,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct OrderBookResponse {
    bids: Vec<OrderLevel>,
    asks: Vec<OrderLevel>,
    timestamp: u64,
}

#[derive(Debug, Deserialize)]
struct OrderLevel {
    price: String,
    size: String,
}

impl OrderbookMonitor {
    pub fn new(config: &Config) -> Self {
        let client = match Client::builder()
            .timeout(Duration::from_millis(config.api.http_timeout_ms.max(100)))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "Failed to build HTTP client with timeout, fallback to default client: {}",
                    e
                );
                Client::new()
            }
        };

        Self {
            config: config.clone(),
            client,
        }
    }

    pub async fn wait_for_clearing(&self, market: &str) -> bool {
        info!("Monitoring orderbook clearing for market: {}", market);

        let start = Instant::now();
        let timeout = Duration::from_secs(self.config.monitor.clearing_timeout_seconds.max(1));
        let poll_interval = Duration::from_millis(self.config.monitor.poll_interval_ms.max(50));

        loop {
            if start.elapsed() > timeout {
                warn!(
                    "Timeout waiting for orderbook clearing in market {}",
                    market
                );
                return false;
            }

            match self.check_orderbook_clear(market).await {
                Ok(cleared) => {
                    if cleared {
                        info!("Orderbook cleared for market: {}", market);
                        return true;
                    }
                }
                Err(e) => {
                    warn!("Failed to check orderbook for market {}: {}", market, e);
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    async fn check_orderbook_clear(&self, market: &str) -> Result<bool> {
        // 构造 CLOB API URL
        let base = self.config.clob.host.trim_end_matches('/');
        let path = if self.config.api.orderbook_path.starts_with('/') {
            self.config.api.orderbook_path.as_str()
        } else {
            "/book"
        };
        let url = format!("{}{}?token_id={}", base, path, market);

        debug!("Checking orderbook: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch orderbook")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Orderbook API error {}: {}", status, body);
        }

        let orderbook: OrderBookResponse = response
            .json()
            .await
            .context("Failed to parse orderbook response")?;

        let best_bid = orderbook
            .bids
            .first()
            .and_then(|l| l.price.parse::<f64>().ok())
            .unwrap_or(0.0);
        let best_ask = orderbook
            .asks
            .first()
            .and_then(|l| l.price.parse::<f64>().ok())
            .unwrap_or(0.0);
        debug!(
            "Orderbook ts={} bids={} asks={} best_bid={:.4} best_ask={:.4}",
            orderbook.timestamp,
            orderbook.bids.len(),
            orderbook.asks.len(),
            best_bid,
            best_ask
        );

        // 检查订单簿是否基本清空（少于3个层级或总深度很小）
        let total_bids: f64 = orderbook
            .bids
            .iter()
            .filter_map(|level| level.size.parse::<f64>().ok())
            .sum();

        let total_asks: f64 = orderbook
            .asks
            .iter()
            .filter_map(|level| level.size.parse::<f64>().ok())
            .sum();

        let is_cleared = self.is_cleared(
            orderbook.bids.len(),
            orderbook.asks.len(),
            total_bids,
            total_asks,
        );

        debug!(
            "Market {} cleared: {} (bids: {}, asks: {})",
            market, is_cleared, total_bids, total_asks
        );

        Ok(is_cleared)
    }

    fn is_cleared(
        &self,
        bid_levels: usize,
        ask_levels: usize,
        total_bids: f64,
        total_asks: f64,
    ) -> bool {
        bid_levels <= self.config.monitor.max_levels_per_side
            && ask_levels <= self.config.monitor.max_levels_per_side
            && total_bids < self.config.monitor.max_depth_per_side
            && total_asks < self.config.monitor.max_depth_per_side
    }
}

#[cfg(test)]
mod tests {
    use super::OrderbookMonitor;
    use crate::config::{
        ApiConfig, ClobConfig, Config, MonitorConfig, PolygonConfig, StrategyConfig, WarningConfig,
    };

    fn test_config() -> Config {
        Config {
            polygon: PolygonConfig {
                rpc_url: "wss://mumbai-rpc.com".to_string(),
                ws_rpc_url: None,
                private_key: secrecy::SecretString::default(),
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
                attack_gas_limit: 50_000,
                attack_nonce_gap: true,
                target_spread_bps: 5_000,
                min_liquidity_usd: 10_000.0,
                exclude_addresses: vec![],
                max_attacks_per_day: 10,
                cooldown_seconds: 300,
            },
            api: ApiConfig::default(),
            monitor: MonitorConfig {
                clearing_timeout_seconds: 30,
                poll_interval_ms: 500,
                max_levels_per_side: 2,
                max_depth_per_side: 100.0,
            },
            warning: WarningConfig {
                testnet_only: true,
                acknowledged: true,
            },
        }
    }

    #[test]
    fn test_is_cleared_true_when_shallow_and_small_depth() {
        let monitor = OrderbookMonitor::new(&test_config());
        assert!(monitor.is_cleared(2, 2, 50.0, 80.0));
    }

    #[test]
    fn test_is_cleared_false_when_levels_too_many() {
        let monitor = OrderbookMonitor::new(&test_config());
        assert!(!monitor.is_cleared(3, 2, 50.0, 80.0));
    }

    #[test]
    fn test_is_cleared_false_when_depth_too_large() {
        let monitor = OrderbookMonitor::new(&test_config());
        assert!(!monitor.is_cleared(2, 2, 120.0, 80.0));
    }
}
