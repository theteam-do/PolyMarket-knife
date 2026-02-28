//! 市场数据客户端

use anyhow::{Context, Result};
use reqwest::Client;
use tracing::info;

use crate::auth::{AuthConfig, AuthMiddleware};
use crate::types::Market;

/// 市场数据客户端
#[derive(Clone)]
pub struct MarketClient {
    client: Client,
    base_url: String,
    auth: Option<AuthMiddleware>,
}

impl MarketClient {
    pub fn new(_host: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            base_url: "https://gamma-api.polymarket.com".to_string(),
            auth: None,
        }
    }

    pub fn with_auth(_host: &str, config: &AuthConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            base_url: "https://gamma-api.polymarket.com".to_string(),
            auth: Some(AuthMiddleware::new(config)),
        }
    }

    /// 获取活跃市场列表
    pub async fn get_active_markets(&self, limit: u32) -> Result<Vec<Market>> {
        let url = format!(
            "{}/markets?active=true&closed=false&limit={}",
            self.base_url,
            limit
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch markets")?;

        if !response.status().is_success() {
            anyhow::bail!("Markets request failed: {}", response.status());
        }

        let markets: Vec<Market> = response
            .json()
            .await
            .context("Failed to parse markets response")?;

        info!("Fetched {} active markets", markets.len());
        Ok(markets)
    }

    /// 根据条件 ID 获取市场
    pub async fn get_market(&self, condition_id: &str) -> Result<Market> {
        let url = format!("{}/markets/{}", self.base_url, condition_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch market")?;

        if !response.status().is_success() {
            anyhow::bail!("Market request failed: {}", response.status());
        }

        let market: Market = response
            .json()
            .await
            .context("Failed to parse market response")?;

        Ok(market)
    }

    /// 根据标签获取市场
    pub async fn get_markets_by_tag(&self, tag: &str, limit: u32) -> Result<Vec<Market>> {
        let url = format!(
            "{}/markets?tag={}&limit={}",
            self.base_url,
            tag,
            limit
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch markets by tag")?;

        if !response.status().is_success() {
            anyhow::bail!("Markets by tag request failed: {}", response.status());
        }

        let markets: Vec<Market> = response
            .json()
            .await
            .context("Failed to parse markets response")?;

        Ok(markets)
    }

    /// 搜索市场
    pub async fn search_markets(&self, query: &str, limit: u32) -> Result<Vec<Market>> {
        let url = format!(
            "{}/search?query={}&limit={}",
            self.base_url,
            query,
            limit
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to search markets")?;

        if !response.status().is_success() {
            anyhow::bail!("Search request failed: {}", response.status());
        }

        let markets: Vec<Market> = response
            .json()
            .await
            .context("Failed to parse search response")?;

        Ok(markets)
    }
}
