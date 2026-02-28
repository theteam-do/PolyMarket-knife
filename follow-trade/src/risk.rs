//! 风控管理器

use crate::config::StrategyConfig;
use crate::monitor::TradeEvent;
use std::collections::HashMap;

pub struct RiskManager {
    config: StrategyConfig,
    daily_pnl: f64,
    pub positions: HashMap<String, f64>,
}

impl RiskManager {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
            daily_pnl: 0.0,
            positions: HashMap::new(),
        }
    }

    pub fn can_trade(&self, trade: &TradeEvent) -> bool {
        if self.config.blacklist.contains(&trade.market) {
            return false;
        }

        if self.daily_pnl < -self.config.max_daily_loss {
            return false;
        }

        let copy_size = trade.size_usd * self.config.copy_ratio;
        if copy_size < self.config.min_trade_size_usd {
            return false;
        }

        true
    }

    pub fn update_position(&mut self, market_id: &str, delta: f64) {
        let pos = self.positions.entry(market_id.to_string()).or_insert(0.0);
        *pos += delta;
    }

    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
    }

    pub fn total_position_value(&self) -> f64 {
        self.positions.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::{Side, TradeEvent};

    fn create_test_risk_manager() -> RiskManager {
        let config = StrategyConfig {
            smart_addresses: vec![],
            min_trade_size_usd: 100.0,
            max_trade_size_usd: 5000.0,
            copy_ratio: 0.1,
            slippage_tolerance: 0.02,
            max_position_per_market: 10000.0,
            max_daily_loss: 1000.0,
            blacklist: vec!["bad_market".to_string()],
        };
        RiskManager::new(&config)
    }

    fn create_test_trade(side: Side, size: f64) -> TradeEvent {
        TradeEvent {
            from: "0xSmartMoney".to_string(),
            market: "test_market".to_string(),
            market_id: "test_id".to_string(),
            side,
            size_usd: size,
            price: 0.50,
            timestamp: 0,
        }
    }

    #[test]
    fn test_can_trade_initial() {
        let risk = create_test_risk_manager();
        let trade = create_test_trade(Side::Buy, 1000.0);
        assert!(risk.can_trade(&trade));
    }

    #[test]
    fn test_min_trade_size() {
        let risk = create_test_risk_manager();

        let trade_small = create_test_trade(Side::Buy, 100.0);
        assert!(!risk.can_trade(&trade_small));

        let trade_ok = create_test_trade(Side::Buy, 1000.0);
        assert!(risk.can_trade(&trade_ok));
    }

    #[test]
    fn test_update_position() {
        let mut risk = create_test_risk_manager();

        risk.update_position("market1", 100.0);

        let pos = risk.positions.get("market1").unwrap();
        assert_eq!(*pos, 100.0);
    }

    #[test]
    fn test_update_pnl() {
        let mut risk = create_test_risk_manager();

        risk.update_pnl(50.0);
        assert_eq!(risk.daily_pnl, 50.0);

        risk.update_pnl(-30.0);
        assert_eq!(risk.daily_pnl, 20.0);
    }

    #[test]
    fn test_reset_daily() {
        let mut risk = create_test_risk_manager();

        risk.update_pnl(100.0);
        risk.reset_daily();

        assert_eq!(risk.daily_pnl, 0.0);
    }
}
