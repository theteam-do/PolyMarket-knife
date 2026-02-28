//! 链上交易监控器

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::{warn, instrument};

use crate::config::Config;

pub struct ChainMonitor {
    client: Client,
    data_api_url: String,
    smart_addresses: Vec<String>,
}

impl ChainMonitor {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
            data_api_url: "https://data-api.polymarket.com".to_string(),
            smart_addresses: config.strategy.smart_addresses.clone(),
        }
    }

    #[instrument(skip(self))]
    pub async fn fetch_trades(&self) -> Result<Vec<TradeEvent>> {
        let mut all_trades = Vec::new();

        // 并行获取每个聪明钱地址的交易
        for address in &self.smart_addresses {
            match self.fetch_address_trades(address).await {
                Ok(trades) => all_trades.extend(trades),
                Err(e) => {
                    warn!("Failed to fetch trades for {}: {}", address, e);
                }
            }
        }

        // 只保留最近的交易 (过去 5 分钟)
        let now = timestamp_sec();
        all_trades.retain(|t| now - t.timestamp < 300);

        Ok(all_trades)
    }

    async fn fetch_address_trades(&self, address: &str) -> Result<Vec<TradeEvent>> {
        let url = format!(
            "{}/activity/user/{}?limit=50",
            self.data_api_url,
            address
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch activity")?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let activity: ActivityResponse = response
            .json()
            .await
            .context("Failed to parse activity response")?;

        Ok(activity
            .activity
            .into_iter()
            .filter_map(|a| {
                if a.action != "trade" {
                    return None;
                }
                
                let side = if a.side.to_lowercase() == "buy" {
                    Side::Buy
                } else {
                    Side::Sell
                };

                Some(TradeEvent {
                    from: address.to_string(),
                    market: a.market_slug.clone(),
                    market_id: a.condition_id.clone(),
                    side,
                    size_usd: a.size_usd,
                    price: a.price,
                    timestamp: a.timestamp,
                })
            })
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct ActivityResponse {
    activity: Vec<ActivityItem>,
}

#[derive(Debug, Deserialize)]
struct ActivityItem {
    action: String,
    side: String,
    size_usd: f64,
    price: f64,
    timestamp: u64,
    market_slug: String,
    condition_id: String,
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

fn timestamp_sec() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
