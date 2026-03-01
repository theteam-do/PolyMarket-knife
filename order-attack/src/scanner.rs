//! 目标扫描器

use crate::config::{ApiConfig, StrategyConfig};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info, warn};

pub struct TargetScanner {
    config: StrategyConfig,
    api: ApiConfig,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct GammaMarket {
    id: String,
    volume: String,
    outcome_prices: Option<Vec<String>>,
}

impl TargetScanner {
    pub fn new(config: &StrategyConfig, api: &ApiConfig) -> Self {
        let client = match Client::builder()
            .timeout(std::time::Duration::from_millis(api.http_timeout_ms.max(100)))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to build HTTP client with timeout, fallback to default client: {}", e);
                Client::new()
            }
        };

        Self {
            config: config.clone(),
            api: api.clone(),
            client,
        }
    }

    pub async fn scan(&self) -> Result<Vec<TargetMarket>> {
        info!("Scanning for target markets...");
        
        // 获取 Polymarket 市场列表
        let markets = self.fetch_polymarket_markets().await?;
        
        // 过滤目标市场
        let mut targets = Vec::new();
        for market in markets {
            if self.is_target_market(&market) {
                targets.push(TargetMarket {
                    market: market.id,
                    liquidity_usd: market.volume.parse().unwrap_or(0.0),
                    has_prices: market.outcome_prices.is_some(),
                });
            }
        }

        info!("Found {} target markets", targets.len());
        Ok(targets)
    }

    async fn fetch_polymarket_markets(&self) -> Result<Vec<GammaMarket>> {
        let url = &self.api.gamma_markets_url;
        
        debug!("Fetching markets from: {}", url);
        
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to fetch markets from Gamma API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Gamma API error {}: {}", status, body);
        }

        let markets: Vec<GammaMarket> = response.json().await
            .context("Failed to parse markets response")?;

        debug!("Fetched {} markets from Gamma API", markets.len());
        Ok(markets)
    }

    fn is_target_market(&self, market: &GammaMarket) -> bool {
        // 检查流动性
        let volume = match market.volume.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                debug!("Market {} filtered: invalid volume format", market.id);
                return false;
            }
        };

        if volume < self.config.min_liquidity_usd {
            debug!("Market {} filtered: insufficient liquidity ${:.2}", market.id, volume);
            return false;
        }

        // 检查是否有价格数据
        if market.outcome_prices.is_none() || market.outcome_prices.as_ref().unwrap().len() < 2 {
            debug!("Market {} filtered: insufficient price data", market.id);
            return false;
        }

        // 检查是否在排除列表中
        if self.config.exclude_addresses.iter().any(|addr| market.id.contains(addr)) {
            debug!("Market {} filtered: in exclude list", market.id);
            return false;
        }

        debug!("Market {} qualifies as target", market.id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{GammaMarket, TargetScanner};
    use crate::config::{ApiConfig, StrategyConfig};

    fn strategy_config() -> StrategyConfig {
        StrategyConfig {
            attack_gas_limit: 21_000,
            attack_nonce_gap: true,
            target_spread_bps: 500,
            min_liquidity_usd: 100.0,
            exclude_addresses: vec!["blocked-market".to_string()],
            max_attacks_per_day: 10,
            cooldown_seconds: 1,
        }
    }

    fn api_config() -> ApiConfig {
        ApiConfig::default()
    }

    #[test]
    fn test_target_filter_rejects_low_liquidity() {
        let scanner = TargetScanner::new(&strategy_config(), &api_config());
        let market = GammaMarket {
            id: "m1".to_string(),
            volume: "50".to_string(),
            outcome_prices: Some(vec!["0.4".to_string(), "0.6".to_string()]),
        };

        assert!(!scanner.is_target_market(&market));
    }

    #[test]
    fn test_target_filter_rejects_excluded_market() {
        let scanner = TargetScanner::new(&strategy_config(), &api_config());
        let market = GammaMarket {
            id: "blocked-market-123".to_string(),
            volume: "1000".to_string(),
            outcome_prices: Some(vec!["0.4".to_string(), "0.6".to_string()]),
        };

        assert!(!scanner.is_target_market(&market));
    }

    #[test]
    fn test_target_filter_accepts_valid_market() {
        let scanner = TargetScanner::new(&strategy_config(), &api_config());
        let market = GammaMarket {
            id: "valid-market".to_string(),
            volume: "1000".to_string(),
            outcome_prices: Some(vec!["0.4".to_string(), "0.6".to_string()]),
        };

        assert!(scanner.is_target_market(&market));
    }
}

#[derive(Debug)]
pub struct TargetMarket {
    pub market: String,
    pub liquidity_usd: f64,
    pub has_prices: bool,
}
