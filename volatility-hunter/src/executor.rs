//! 订单执行器 - 使用 poly-client

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use tracing::{info, instrument};

use poly_client::{PolyClient, Side as PolySide, OrderType};
use crate::config::Config;
use crate::signal::Signal;

pub struct Executor {
    client: PolyClient,
    config: Config,
}

impl Executor {
    pub fn new(config: &Config) -> Self {
        let client = if config.polygon.private_key.is_empty() {
            PolyClient::new(&config.clob.host)
        } else {
            PolyClient::with_auth(&config.clob.host, &config.to_auth_config())
        };

        Self {
            client,
            config: config.clone(),
        }
    }

    #[instrument(skip(self), fields(signal = ?signal))]
    pub async fn execute(&self, signal: &Signal) -> Result<()> {
        let position = self.calculate_position(signal.confidence());
        
        info!(
            "Executing order: {} {} @ confidence {:.2}, position ${}",
            match signal {
                Signal::Buy { .. } => "BUY",
                Signal::Sell { .. } => "SELL",
            },
            signal.symbol(),
            signal.confidence(),
            position
        );

        // TODO: 在 Polymarket CLOB 下单
        // 需要将加密货币价格映射到 Polymarket 市场
        
        Ok(())
    }

    fn calculate_position(&self, confidence: f64) -> Decimal {
        let base = Decimal::from_f64_retain(self.config.strategy.base_position_usd).unwrap();
        let max = Decimal::from_f64_retain(self.config.strategy.max_position_usd).unwrap();
        
        // 高置信度用大仓位，低置信度用小仓位
        if confidence >= self.config.strategy.confidence_high {
            // 高置信度：最大仓位
            max
        } else if confidence >= 0.6 {
            // 中等置信度：中等仓位
            max * Decimal::from_f64_retain(0.3).unwrap()
        } else {
            // 低置信度：基础仓位 (埋伏)
            base
        }
    }
}
