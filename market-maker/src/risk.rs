//! 风控管理器

use crate::config::RiskConfig;

pub struct RiskManager {
    config: RiskConfig,
    can_trade_flag: bool,
    daily_pnl: f64,
}

impl RiskManager {
    pub fn new(config: &RiskConfig) -> Self {
        Self {
            config: config.clone(),
            can_trade_flag: true,
            daily_pnl: 0.0,
        }
    }

    pub fn can_trade(&self) -> bool {
        self.can_trade_flag && self.daily_pnl > -self.config.max_loss_per_day
    }

    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
    }

    pub fn stop_trading(&mut self) {
        self.can_trade_flag = false;
    }

    pub fn resume_trading(&mut self) {
        self.can_trade_flag = true;
    }
}
