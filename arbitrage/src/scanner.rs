//! 市场扫描器 - 从 Polymarket API 获取市场数据

use anyhow::{Context, Result};
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{info, warn};

use crate::config::Config;

/// Polymarket Gamma API Event 响应
#[derive(Debug, Clone, Deserialize)]
struct GammaEvent {
    markets: Vec<GammaMarket>,
}

/// Polymarket Gamma API Market 响应
#[derive(Debug, Clone, Deserialize)]
struct GammaMarket {
    #[serde(rename = "question")]
    market_id: String,
    outcomes: Option<String>,
    #[serde(rename = "outcomePrices")]
    outcome_prices: Option<String>,
    #[serde(rename = "clobTokenIds")]
    clob_token_ids: Option<String>,
    #[serde(rename = "volume")]
    volume: Option<String>,
}

pub struct Scanner {
    config: Config,
    client: Client,
}

impl Scanner {
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

    /// 扫描市场，获取价格数据
    pub async fn scan(&self) -> Result<Vec<MarketPrice>> {
        // 从 Polymarket Gamma API 获取市场数据
        // API: https://gamma-api.polymarket.com/events?closed=false&active=true
        let url = "https://gamma-api.polymarket.com/events?closed=false&active=true&limit=100";
        
        info!("Scanning markets from: {}", url);
        
        match self.fetch_markets(url).await {
            Ok(mut markets) => {
                markets.retain(|m| {
                    !self
                        .config
                        .strategy
                        .exclude_market_ids
                        .iter()
                        .any(|id| id == &m.market_id)
                });
                if !self.config.strategy.include_all {
                    markets.truncate(50);
                }
                info!("Found {} active markets", markets.len());
                Ok(markets)
            }
            Err(e) => {
                warn!("Failed to fetch from API: {}. Using fallback data.", e);
                // 返回备用数据
                let mut markets = self.fallback_markets();
                markets.retain(|m| {
                    !self
                        .config
                        .strategy
                        .exclude_market_ids
                        .iter()
                        .any(|id| id == &m.market_id)
                });
                Ok(markets)
            }
        }
    }

    /// 从 API 获取市场数据
    async fn fetch_markets(&self, url: &str) -> Result<Vec<MarketPrice>> {
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;
        
        let events: Vec<GammaEvent> = response
            .json()
            .await
            .context("Failed to parse response")?;
        
        let mut result = Vec::new();
        
        for event in events {
            for market in event.markets {
                if let (Some(outcomes), Some(outcome_prices), Some(clob_token_ids)) = (
                    market.outcomes,
                    market.outcome_prices,
                    market.clob_token_ids,
                ) {
                    let parsed_outcomes: Vec<String> = serde_json::from_str(&outcomes).unwrap_or_default();
                    let parsed_prices: Vec<String> = serde_json::from_str(&outcome_prices).unwrap_or_default();
                    let parsed_tokens: Vec<String> = serde_json::from_str(&clob_token_ids).unwrap_or_default();

                    let mut yes_idx = None;
                    let mut no_idx = None;

                    for (i, outcome) in parsed_outcomes.iter().enumerate() {
                        let lower = outcome.to_lowercase();
                        if lower.contains("yes") || lower.contains("for") {
                            yes_idx = Some(i);
                        } else if lower.contains("no") || lower.contains("against") {
                            no_idx = Some(i);
                        }
                    }

                    if let (Some(y), Some(n)) = (yes_idx, no_idx) {
                        if y < parsed_tokens.len() && n < parsed_tokens.len() && y < parsed_prices.len() && n < parsed_prices.len() {
                            let yes_price = Decimal::from_str(&parsed_prices[y]).unwrap_or(dec!(0.5));
                            let no_price = Decimal::from_str(&parsed_prices[n]).unwrap_or(dec!(0.5));
                            let volume = market.volume.and_then(|v| Decimal::from_str(&v).ok()).unwrap_or(Decimal::ZERO);

                            result.push(MarketPrice {
                                market_id: market.market_id.clone(),
                                token_id_yes: parsed_tokens[y].clone(),
                                token_id_no: parsed_tokens[n].clone(),
                                yes_price,
                                no_price,
                                volume_24h: volume,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(result)
    }

    /// 备用市场数据（API 失败时使用）
    fn fallback_markets(&self) -> Vec<MarketPrice> {
        // Fallback needs real token ids to pass U256 parsing, otherwise it will be skipped
        vec![
            MarketPrice {
                market_id: "MicroStrategy sells any Bitcoin in 2025?".to_string(),
                token_id_yes: "93592949212798121127213117304912625505836768562433217537850469496310204567695".to_string(),
                token_id_no: "3074539347152748632858978545166555332546941892131779352477699494423276162345".to_string(),
                yes_price: Decimal::from_f64_retain(0.45).unwrap(),
                no_price: Decimal::from_f64_retain(0.48).unwrap(),
                volume_24h: Decimal::from_f64_retain(100000.0).unwrap(),
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct MarketPrice {
    pub market_id: String,
    pub token_id_yes: String,
    pub token_id_no: String,
    pub yes_price: Decimal,
    pub no_price: Decimal,
    pub volume_24h: Decimal,
}
