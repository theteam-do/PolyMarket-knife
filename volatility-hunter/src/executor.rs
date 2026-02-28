//! 订单执行器 - 简化版本

use anyhow::Result;
use rust_decimal::Decimal;
use tracing::{info, instrument};

use crate::config::Config;
use crate::signal::Signal;

pub struct Executor {
    config: Config,
}

impl Executor {
    pub fn new(config: &Config) -> Self {
        Self {
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

        // TODO: 使用官方 SDK 下单
        Ok(())
    }

    fn calculate_position(&self, confidence: f64) -> Decimal {
        let base = Decimal::from_f64_retain(self.config.strategy.base_position_usd).unwrap();
        let max = Decimal::from_f64_retain(self.config.strategy.max_position_usd).unwrap();
        
        if confidence >= self.config.strategy.confidence_high {
            max
        } else if confidence >= 0.6 {
            max * Decimal::from_f64_retain(0.3).unwrap()
        } else {
            base
        }
    }
}
