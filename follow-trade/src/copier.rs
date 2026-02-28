//! 交易复制器 - 简化版本

use anyhow::Result;
use rust_decimal::Decimal;
use tracing::{info, instrument};

use crate::config::Config;
use crate::monitor::{TradeEvent, Side};

pub struct TradeCopier {
    config: Config,
}

impl TradeCopier {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    #[instrument(skip(self), fields(trade = ?trade))]
    pub async fn copy(&self, trade: &TradeEvent) -> Result<()> {
        let size = self.calculate_copy_size(trade.size_usd);
        
        info!(
            "Copying trade: {} ${} (slippage check skipped)",
            if trade.side == Side::Buy { "BUY" } else { "SELL" },
            size
        );

        // TODO: 使用官方 SDK 下单
        Ok(())
    }

    fn calculate_copy_size(&self, original_size: f64) -> Decimal {
        let copy_ratio = Decimal::from_f64_retain(self.config.strategy.copy_ratio).unwrap();
        let size = Decimal::from_f64_retain(original_size).unwrap() * copy_ratio;
        
        let min_size = Decimal::from_f64_retain(self.config.strategy.min_trade_size_usd).unwrap();
        let max_size = Decimal::from_f64_retain(self.config.strategy.max_trade_size_usd).unwrap();
        
        size.clamp(min_size, max_size)
    }
}
