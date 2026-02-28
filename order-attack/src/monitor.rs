//! 订单簿监控器

use anyhow::Result;
use reqwest::Client;
use tracing::{instrument};

use crate::config::Config;

pub struct OrderbookMonitor {
    client: Client,
    config: Config,
}

impl OrderbookMonitor {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
            config: config.clone(),
        }
    }

    #[instrument(skip(self), fields(market = %market))]
    pub async fn wait_for_clearing(&self, market: &str) -> bool {
        // 等待订单簿被清空
        // 最多等待 10 秒
        
        for i in 0..10 {
            match self.fetch_orderbook(market).await {
                Some(ob) => {
                    if ob.is_empty {
                        tracing::info!("Orderbook cleared after {} seconds", i);
                        return true;
                    }
                }
                None => {
                    tracing::warn!("Failed to fetch orderbook");
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        tracing::warn!("Orderbook did not clear within timeout");
        false
    }

    async fn fetch_orderbook(&self, _market: &str) -> Option<Orderbook> {
        // TODO: 从 CLOB API 获取订单簿
        
        Some(Orderbook {
            is_empty: false,
            bids: vec![],
            asks: vec![],
        })
    }
}

#[derive(Debug)]
pub struct Orderbook {
    pub is_empty: bool,
    pub bids: Vec<(f64, f64)>, // (price, size)
    pub asks: Vec<(f64, f64)>,
}
