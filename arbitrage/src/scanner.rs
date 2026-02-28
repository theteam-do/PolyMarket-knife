//! 市场扫描器 - 从 Polymarket API 获取市场数据

use anyhow::{Context, Result};
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{info, warn};

use crate::config::Config;

/// Polymarket Gamma API 响应
#[derive(Debug, Clone, Deserialize)]
struct GammaMarket {
    #[serde(rename = "question")]
    market_id: String,
    #[serde(rename = "outcomeTokens")]
    outcome_tokens: Vec<OutcomeToken>,
    #[serde(rename = "volume")]
    volume: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OutcomeToken {
    #[serde(rename = "id")]
    token_id: String,
    #[serde(rename = "price")]
    price: String,
    #[serde(rename = "outcome")]
    outcome: String,
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
        
        let markets: Vec<GammaMarket> = response
            .json()
            .await
            .context("Failed to parse response")?;
        
        let mut result = Vec::new();
        
        for market in markets {
            // 查找 Yes 和 No 代币
            let yes_token = market.outcome_tokens.iter()
                .find(|t| t.outcome.to_lowercase().contains("yes") || t.outcome.to_lowercase().contains("for"));
            let no_token = market.outcome_tokens.iter()
                .find(|t| t.outcome.to_lowercase().contains("no") || t.outcome.to_lowercase().contains("against"));
            
            if let (Some(yes), Some(no)) = (yes_token, no_token) {
                let yes_price = Decimal::from_str(&yes.price).unwrap_or(dec!(0.5));
                let no_price = Decimal::from_str(&no.price).unwrap_or(dec!(0.5));
                let volume = Decimal::from_str(&market.volume).unwrap_or(Decimal::ZERO);
                
                result.push(MarketPrice {
                    market_id: market.market_id.clone(),
                    token_id_yes: yes.token_id.clone(),
                    token_id_no: no.token_id.clone(),
                    yes_price,
                    no_price,
                    volume_24h: volume,
                });
            }
        }
        
        Ok(result)
    }

    /// 备用市场数据（API 失败时使用）
    fn fallback_markets(&self) -> Vec<MarketPrice> {
        vec![
            MarketPrice {
                market_id: "Will Trump win 2024 election?".to_string(),
                token_id_yes: "0x1234...yes".to_string(),
                token_id_no: "0x1234...no".to_string(),
                yes_price: Decimal::from_f64_retain(0.45).unwrap(),
                no_price: Decimal::from_f64_retain(0.48).unwrap(),
                volume_24h: Decimal::from_f64_retain(100000.0).unwrap(),
            },
            MarketPrice {
                market_id: "Will BTC reach $100k in 2024?".to_string(),
                token_id_yes: "0x5678...yes".to_string(),
                token_id_no: "0x5678...no".to_string(),
                yes_price: Decimal::from_f64_retain(0.35).unwrap(),
                no_price: Decimal::from_f64_retain(0.58).unwrap(),
                volume_24h: Decimal::from_f64_retain(50000.0).unwrap(),
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
