//! 风控管理器

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::config::StrategyConfig;
use crate::monitor::TradeEvent;

pub struct RiskManager {
    config: StrategyConfig,
    daily_pnl: Decimal,
}

impl RiskManager {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
            daily_pnl: dec!(0),
        }
    }

    pub fn can_trade(&self, trade: &TradeEvent) -> bool {
        // 检查黑名单
        if self.config.blacklist.contains(&trade.market) {
            return false;
        }

        // 检查日亏损
        if self.daily_pnl < -Decimal::from_f64_retain(self.config.max_daily_loss).unwrap() {
            return false;
        }

        // 检查最小交易规模
        let copy_size = trade.size_usd * self.config.copy_ratio;
        if copy_size < self.config.min_trade_size_usd {
            return false;
        }

        true
    }

    pub fn update_pnl(&mut self, pnl: Decimal) {
        self.daily_pnl += pnl;
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = dec!(0);
    }
}
