//! 风控管理器

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::config::StrategyConfig;
use crate::signal::Signal;

pub struct RiskManager {
    config: StrategyConfig,
    daily_pnl: Decimal,
    consecutive_losses: u32,
}

impl RiskManager {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
            daily_pnl: dec!(0),
            consecutive_losses: 0,
        }
    }

    pub fn can_trade(&self, signal: &Signal) -> bool {
        // 检查日亏损
        let max_loss = Decimal::from_f64_retain(self.config.max_daily_loss).unwrap();
        if self.daily_pnl < -max_loss {
            return false;
        }

        // 检查连续亏损
        if self.consecutive_losses >= 5 {
            return false;
        }

        // 高置信度信号总是可以交易
        if signal.confidence() >= self.config.confidence_high {
            return true;
        }

        // 低置信度信号需要更严格检查
        signal.confidence() >= 0.5
    }

    pub fn update_pnl(&mut self, pnl: Decimal) {
        self.daily_pnl += pnl;
        
        if pnl < dec!(0) {
            self.consecutive_losses += 1;
        } else {
            self.consecutive_losses = 0;
        }
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = dec!(0);
        self.consecutive_losses = 0;
    }
}
