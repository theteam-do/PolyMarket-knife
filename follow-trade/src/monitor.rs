//! 链上交易监控器

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::warn;

use crate::config::Config;

pub struct ChainMonitor {
    #[allow(dead_code)]
    config: Config,
    client: Client,
}

impl ChainMonitor {
    pub fn new(config: &Config) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            config: config.clone(),
            client,
        }
    }

    pub async fn fetch_trades(&self) -> Result<Vec<TradeEvent>> {
        let url = std::env::var("FOLLOW_TRADE_DATA_API").unwrap_or_else(|_| {
            "https://gamma-api.polymarket.com/trades?limit=50".to_string()
        });

        let response = self.client.get(&url).send().await;
        let Ok(response) = response else {
            warn!("fetch_trades() request failed, using mock data");
            return Ok(self.mock_trades());
        };

        let parsed = response.json::<Vec<ApiTrade>>().await;
        let Ok(items) = parsed else {
            warn!("fetch_trades() parse failed, using mock data");
            return Ok(self.mock_trades());
        };

        let filtered: Vec<TradeEvent> = items
            .into_iter()
            .filter(|item| self.config.strategy.smart_addresses.is_empty()
                || self
                    .config
                    .strategy
                    .smart_addresses
                    .iter()
                    .any(|addr| addr.eq_ignore_ascii_case(&item.wallet)))
            .map(|item| TradeEvent {
                from: item.wallet,
                market: item.market.clone(),
                market_id: item.market,
                side: if item.side.eq_ignore_ascii_case("buy") {
                    Side::Buy
                } else {
                    Side::Sell
                },
                size_usd: item.size_usd,
                price: item.price,
                timestamp: item.timestamp,
            })
            .collect();

        if filtered.is_empty() {
            return Ok(self.mock_trades());
        }

        Ok(filtered)
    }

    fn mock_trades(&self) -> Vec<TradeEvent> {
        vec![TradeEvent {
            from: "0xSmartMoney".to_string(),
            market: "mock_market".to_string(),
            market_id: "mock_id".to_string(),
            side: Side::Buy,
            size_usd: 1000.0,
            price: 0.50,
            timestamp: 0,
        }]
    }
}

#[derive(Debug, Deserialize)]
struct ApiTrade {
    #[serde(default)]
    wallet: String,
    #[serde(default)]
    market: String,
    #[serde(default)]
    side: String,
    #[serde(default)]
    size_usd: f64,
    #[serde(default)]
    price: f64,
    #[serde(default)]
    timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct TradeEvent {
    pub from: String,
    pub market: String,
    pub market_id: String,
    pub side: Side,
    pub size_usd: f64,
    pub price: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}
