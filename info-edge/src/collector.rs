//! 新闻收集器

use crate::config::SourcesConfig;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct NewsItem {
    pub source: String,
    pub title: String,
    pub content: String,
    pub timestamp: u64,
    pub market: Option<String>,
}

pub struct NewsCollector {
    sources: SourcesConfig,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct NewsApiResponse {
    articles: Vec<Article>,
}

#[derive(Debug, Deserialize)]
struct Article {
    title: String,
    description: Option<String>,
    url: Option<String>,
    published_at: Option<String>,
    source: Option<String>,
}

impl NewsCollector {
    pub fn new(sources: &SourcesConfig, proxy_url: Option<&str>) -> Self {
        let mut client_builder = Client::builder().timeout(std::time::Duration::from_secs(10));
        if let Some(proxy_url) = proxy_url {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }
        let client = match client_builder.build() {
            Ok(client) => client,
            Err(e) => {
                warn!(
                    "Failed to build HTTP client with timeout, fallback to default client: {}",
                    e
                );
                Client::new()
            }
        };

        Self {
            sources: sources.clone(),
            client,
        }
    }

    pub fn source_count(&self) -> usize {
        self.sources.news_apis.len() + self.sources.gov_websites.len()
    }

    pub async fn fetch(&self) -> Result<Vec<crate::collector::NewsItem>> {
        info!("Fetching news from {} sources...", self.source_count());

        let mut all_news = Vec::new();
        for api_config in &self.sources.news_apis {
            match self.fetch_from_api(api_config).await {
                Ok(mut news) => {
                    info!("Fetched {} news from {}", news.len(), api_config.name);
                    all_news.append(&mut news);
                }
                Err(e) => {
                    warn!("Failed to fetch from API {}: {}", api_config.name, e);
                }
            }
        }

        if all_news.is_empty() {
            warn!("No actionable real news fetched from configured sources");
        }

        info!("Total news items collected: {}", all_news.len());
        Ok(all_news)
    }

    async fn fetch_from_api(
        &self,
        api_config: &crate::config::NewsApiConfig,
    ) -> Result<Vec<crate::collector::NewsItem>> {
        let url = &api_config.url;
        debug!("Fetching news from API: {}", url);

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", api_config.token))
            .send()
            .await
            .context("Failed to send request to news API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("News API error {}: {}", status, body);
        }

        let api_response: NewsApiResponse = response
            .json()
            .await
            .context("Failed to parse news API response")?;

        let mut news_items = Vec::new();
        for article in api_response.articles {
            let content = format!(
                "{} {} {}",
                article.title,
                article.description.as_deref().unwrap_or(""),
                article.url.as_deref().unwrap_or("")
            );

            if self.contains_keywords(&content) {
                let timestamp =
                    parse_published_at(article.published_at.as_deref()).unwrap_or_else(now_ts);

                news_items.push(crate::collector::NewsItem {
                    source: article.source.unwrap_or_else(|| api_config.name.clone()),
                    title: article.title,
                    content: content.clone(),
                    timestamp,
                    market: self.extract_market(&content),
                });
            }
        }

        Ok(news_items)
    }

    fn contains_keywords(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.sources
            .keywords
            .iter()
            .any(|keyword| text_lower.contains(&keyword.to_lowercase()))
    }

    fn extract_market(&self, text: &str) -> Option<String> {
        extract_token_id(text)
    }
}

fn extract_token_id(text: &str) -> Option<String> {
    let hex = Regex::new(r"0x[a-fA-F0-9]{64}").expect("hex token id regex should compile");
    if let Some(matched) = hex.find(text) {
        return Some(matched.as_str().to_ascii_lowercase());
    }

    let keyed_decimal = Regex::new(r"(?i)(?:asset_id|token_id|market_id)[=: /]+([0-9]{10,})")
        .expect("decimal token id regex should compile");
    keyed_decimal
        .captures(text)
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn parse_published_at(input: Option<&str>) -> Option<u64> {
    let raw = input?.trim();
    if raw.is_empty() {
        return None;
    }

    if let Ok(unix) = raw.parse::<u64>() {
        return Some(unix);
    }

    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc).timestamp() as u64)
}

#[cfg(test)]
mod tests {
    use super::{extract_token_id, parse_published_at};

    #[test]
    fn test_parse_published_at_unix() {
        assert_eq!(parse_published_at(Some("1700000000")), Some(1_700_000_000));
    }

    #[test]
    fn test_parse_published_at_rfc3339() {
        assert_eq!(
            parse_published_at(Some("2024-01-02T03:04:05Z")),
            Some(1_704_164_645)
        );
    }

    #[test]
    fn test_parse_published_at_invalid() {
        assert_eq!(parse_published_at(Some("not-a-date")), None);
        assert_eq!(parse_published_at(Some("   ")), None);
        assert_eq!(parse_published_at(None), None);
    }

    #[test]
    fn test_extract_hex_token_id() {
        let token = extract_token_id(
            "Breaking: market token 0x00000000000000000000000000000000000000000000000000000000000000aa surged",
        );
        assert_eq!(
            token.as_deref(),
            Some("0x00000000000000000000000000000000000000000000000000000000000000aa")
        );
    }

    #[test]
    fn test_extract_keyed_decimal_token_id() {
        let token = extract_token_id("asset_id=12345678901234567890");
        assert_eq!(token.as_deref(), Some("12345678901234567890"));
    }

    #[test]
    fn test_extract_token_id_requires_token_pattern() {
        assert_eq!(extract_token_id("general market discussion"), None);
    }
}
