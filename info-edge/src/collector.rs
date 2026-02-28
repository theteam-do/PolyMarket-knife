//! 新闻收集器

use anyhow::Result;
use reqwest::Client;
use tracing::{warn, instrument};

use crate::config::SourcesConfig;

pub struct NewsCollector {
    client: Client,
    sources: SourcesConfig,
}

impl NewsCollector {
    pub fn new(sources: &SourcesConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
            sources: sources.clone(),
        }
    }

    pub fn source_count(&self) -> usize {
        self.sources.news_apis.len() + self.sources.gov_websites.len()
    }

    #[instrument(skip(self))]
    pub async fn fetch(&self) -> Result<Vec<NewsItem>> {
        let mut all_news = Vec::new();

        // 并行抓取所有新闻源
        for api in &self.sources.news_apis {
            match self.fetch_from_api(api).await {
                Ok(items) => all_news.extend(items),
                Err(e) => {
                    warn!("Failed to fetch from {}: {}", api.name, e);
                }
            }
        }

        // 抓取政府网站
        for website in &self.sources.gov_websites {
            match self.fetch_website(website).await {
                Ok(items) => all_news.extend(items),
                Err(e) => {
                    warn!("Failed to fetch from {}: {}", website, e);
                }
            }
        }

        Ok(all_news)
    }

    async fn fetch_from_api(&self, api: &crate::config::NewsApiConfig) -> Result<Vec<NewsItem>> {
        // TODO: 实现各 API 的抓取逻辑
        // Twitter API, Reuters API 等
        
        Ok(vec![])
    }

    async fn fetch_website(&self, _url: &str) -> Result<Vec<NewsItem>> {
        // TODO: 实现网站抓取
        // 使用 reqwest + HTML 解析
        
        Ok(vec![])
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
