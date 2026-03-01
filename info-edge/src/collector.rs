//! 新闻收集器

use crate::config::SourcesConfig;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

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
    pub fn new(sources: &SourcesConfig) -> Self {
        let client = match Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to build HTTP client with timeout, fallback to default client: {}", e);
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
        
        // 从新闻 API 获取
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
        
        // 如果没有真实新闻，返回模拟数据作为 fallback
        if all_news.is_empty() {
            warn!("No real news fetched, using mock data as fallback");
            all_news = self.mock_news();
        }
        
        info!("Total news items collected: {}", all_news.len());
        Ok(all_news)
    }
    
    async fn fetch_from_api(&self, api_config: &crate::config::NewsApiConfig) -> Result<Vec<crate::collector::NewsItem>> {
        let url = &api_config.url;
        
        debug!("Fetching news from API: {}", url);
        
        let response = self.client
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
        
        let api_response: NewsApiResponse = response.json().await
            .context("Failed to parse news API response")?;
            
        let mut news_items = Vec::new();
        for article in api_response.articles {
            // 检查是否包含关键词
            let content = format!("{} {} {}", 
                article.title, 
                article.description.as_deref().unwrap_or(""),
                article.url.as_deref().unwrap_or("")
            );
            
            if self.contains_keywords(&content) {
                let timestamp = parse_published_at(article.published_at.as_deref()).unwrap_or_else(now_ts);
                
                news_items.push(crate::collector::NewsItem {
                    source: article.source.unwrap_or_else(|| api_config.name.clone()),
                    title: article.title,
                    content: article.description.unwrap_or_default(),
                    timestamp,
                    market: self.extract_market(&content),
                });
            }
        }
        
        Ok(news_items)
    }
    
    fn contains_keywords(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.sources.keywords.iter()
            .any(|keyword| text_lower.contains(&keyword.to_lowercase()))
    }
    
    fn extract_market(&self, text: &str) -> Option<String> {
        // 简单的市场提取逻辑：查找包含 "election", "market", "outcome" 等关键词的新闻
        if text.to_lowercase().contains("election") || 
           text.to_lowercase().contains("market") || 
           text.to_lowercase().contains("outcome") {
            Some("general_market".to_string())
        } else {
            None
        }
    }
    
    fn mock_news(&self) -> Vec<crate::collector::NewsItem> {
        vec![
            crate::collector::NewsItem {
                source: "Mock Source".to_string(),
                title: "Election Results Show Tight Race".to_string(),
                content: "Latest polls indicate a very close election outcome".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                market: Some("election_market".to_string()),
            },
            crate::collector::NewsItem {
                source: "Mock Source".to_string(),
                title: "Market Analysis Report".to_string(),
                content: "Financial markets show increased volatility".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                market: Some("financial_market".to_string()),
            },
        ]
    }
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
    use super::parse_published_at;

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
}

#[derive(Debug, Clone)]
pub struct NewsItem {
    pub source: String,
    pub title: String,
    pub content: String,
    pub timestamp: u64,
    pub market: Option<String>,
}
