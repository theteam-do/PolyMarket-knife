//! 风控管理器

use crate::config::RiskConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Position {
    pub yes_size: f64,
    pub no_size: f64,
    pub avg_cost: f64,
}

pub struct RiskManager {
    config: RiskConfig,
    can_trade_flag: bool,
    positions: HashMap<String, Position>,
    daily_pnl: f64,
}

impl RiskManager {
    pub fn new(config: &RiskConfig) -> Self {
        Self {
            config: config.clone(),
            can_trade_flag: true,
            positions: HashMap::new(),
            daily_pnl: 0.0,
        }
    }

    pub fn can_trade(&self) -> bool {
        self.can_trade_flag && self.daily_pnl > -self.config.max_loss_per_day
    }

    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
    }

    pub fn daily_pnl(&self) -> f64 {
        self.daily_pnl
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_risk_manager() -> RiskManager {
        let config = RiskConfig {
            max_position_usd: 10000.0,
            max_loss_per_day: 500.0,
            stop_loss_pct: 5.0,
            max_orders: 10,
            max_order_size_usd: 5000.0,
        };
        RiskManager::new(&config)
    }

    #[test]
    fn test_can_trade_initial() {
        let risk = create_test_risk_manager();
        assert!(risk.can_trade());
    }

    #[test]
    fn test_can_trade_after_loss() {
        let mut risk = create_test_risk_manager();

        risk.update_pnl(-400.0);
        assert!(risk.can_trade());

        risk.update_pnl(-150.0);
        assert!(!risk.can_trade());
    }

    #[test]
    fn test_update_pnl() {
        let mut risk = create_test_risk_manager();

        risk.update_pnl(100.0);
        assert_eq!(risk.daily_pnl(), 100.0);

        risk.update_pnl(-50.0);
        assert_eq!(risk.daily_pnl(), 50.0);
    }

    #[test]
    fn test_reset_daily() {
        let mut risk = create_test_risk_manager();

        risk.update_pnl(-100.0);
        risk.reset_daily();

        assert_eq!(risk.daily_pnl(), 0.0);
    }

    #[test]
    fn test_stop_resume_trading() {
        let mut risk = create_test_risk_manager();

        risk.stop_trading();
        assert!(!risk.can_trade());

        risk.resume_trading();
        assert!(risk.can_trade());
    }
}
