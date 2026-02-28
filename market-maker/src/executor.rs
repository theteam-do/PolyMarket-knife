//! 订单执行器 - 简化版本

use anyhow::Result;
use tracing::warn;

use crate::config::Config;

/// 订单执行器（简化版本）
#[derive(Clone)]
pub struct Executor {
    #[allow(dead_code)]
    config: Config,
}

impl Executor {
    /// 创建新的执行器
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// 获取订单簿
    pub async fn fetch_orderbook(&self, _token_id: &str) -> Result<()> {
        warn!("fetch_orderbook not implemented");
        Ok(())
    }

    /// 下双边订单
    pub async fn place_orders(&self, _token_id: &str, _bid_price: f64, _ask_price: f64) -> Result<()> {
        warn!("place_orders not implemented");
        Ok(())
    }

    /// 取消订单
    pub async fn cancel_orders(&self, _market_id: &str) -> Result<()> {
        warn!("cancel_orders not implemented");
        Ok(())
    }

    /// 取消所有订单
    pub async fn cancel_all_orders(&self) -> Result<()> {
        warn!("cancel_all_orders not implemented");
        Ok(())
    }
}
