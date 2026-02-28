//! 订单执行器 - 简化版本

use anyhow::Result;
use rust_decimal::Decimal;
use tracing::{warn, instrument};

use crate::config::Config;

#[derive(Clone)]
pub struct Executor {
    #[allow(dead_code)]
    config: Config,
}

impl Executor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    #[instrument(skip(self))]
    pub async fn fetch_orderbook(&self, _token_id: &str) -> Result<()> {
        warn!("fetch_orderbook not implemented");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn place_orders(&self, _token_id: &str, _bid_price: f64, _ask_price: f64) -> Result<()> {
        warn!("place_orders not implemented");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cancel_orders(&self, _market_id: &str) -> Result<()> {
        warn!("cancel_orders not implemented");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cancel_all_orders(&self) -> Result<()> {
        warn!("cancel_all_orders not implemented");
        Ok(())
    }
}
