//! 市场扫描器 - 使用 poly-client

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use tracing::{warn, instrument};

use crate::config::Config;

pub struct Scanner {
    client: PolyClient,
    gamma_client: reqwest::Client,
    exclude_ids: Vec<String>,
}

impl Scanner {
    pub fn new(config: &Config) -> Self {
        Self {
            client: PolyClient::new(&config.clob.host),
            gamma_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
            exclude_ids: config.strategy.exclude_market_ids.clone(),
        }
    }

    #[instrument(skip(self))]
    pub async fn scan(&self) -> Result<Vec<MarketPrice>> {
        // 1. 获取所有活跃市场
        let markets = self.fetch_active_markets().await?;

        // 2. 并行获取每个市场的价格
        let mut prices = Vec::new();
        
        for market in markets {
            if self.exclude_ids.contains(&market.condition_id) {
                continue;
            }

            match self.fetch_market_prices(&market).await {
                Ok(price) => prices.push(price),
                Err(e) => {
                    warn!("Failed to fetch prices for {}: {}", market.question, e);
                }
            }
        }

        Ok(prices)
    }

    async fn fetch_active_markets(&self) -> Result<Vec<GammaMarket>> {
        let url = format!(
            "{}/markets?active=true&closed=false&limit=100",
            "https://gamma-api.polymarket.com"
        );

        let response = self.gamma_client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch markets")?;

        if !response.status().is_success() {
            anyhow::bail!("Markets request failed: {}", response.status());
        }

        let markets: Vec<GammaMarket> = response
            .json()
            .await
            .context("Failed to parse markets response")?;

        Ok(markets)
    }

    async fn fetch_market_prices(&self, market: &GammaMarket) -> Result<MarketPrice> {
        // 从 CLOB API 获取 Yes/No 价格
        let yes_price = self.fetch_token_price(&market.clob_token_ids[0]).await?;
        let no_price = if market.clob_token_ids.len() > 1 {
            self.fetch_token_price(&market.clob_token_ids[1]).await?
        } else {
            Decimal::ZERO
        };

        Ok(MarketPrice {
            market_id: market.condition_id.clone(),
            token_id_yes: market.clob_token_ids[0].clone(),
            token_id_no: market.clob_token_ids.get(1).cloned().unwrap_or_default(),
            yes_price,
            no_price,
            volume_24h: market.volume_24h,
        })
    }

    async fn fetch_token_price(&self, token_id: &str) -> Result<Decimal> {
        if token_id.is_empty() {
            return Ok(Decimal::ZERO);
        }

        match self.client.get_orderbook(token_id).await {
            Ok(ob) => {
                if let Some(mid) = ob.mid_price() {
                    Ok(mid)
                } else {
                    Ok(Decimal::ZERO)
                }
            }
            Err(_) => Ok(Decimal::ZERO),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GammaMarket {
    condition_id: String,
    question: String,
    #[serde(rename = "clobTokenIds")]
    clob_token_ids: Vec<String>,
    #[serde(rename = "volume24h")]
    volume_24h: Decimal,
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
