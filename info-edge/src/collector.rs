//! 新闻收集器

use crate::config::SourcesConfig;
use anyhow::Result;

pub struct NewsCollector {
    #[allow(dead_code)]
    sources: SourcesConfig,
}

impl NewsCollector {
    pub fn new(sources: &SourcesConfig) -> Self {
        Self {
            sources: sources.clone(),
        }
    }

    pub fn source_count(&self) -> usize {
        self.sources.news_apis.len() + self.sources.gov_websites.len()
    }

    pub async fn fetch(&self) -> Result<Vec<crate::collector::NewsItem>> {
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
