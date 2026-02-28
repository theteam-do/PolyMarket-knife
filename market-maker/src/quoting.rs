//! 报价引擎

use crate::config::StrategyConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct Quoter {
    spread_bps: f64,
    order_size: f64,
    skew_enabled: bool,
    min_spread_bps: f64,
    max_spread_bps: f64,
}

impl Quoter {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            spread_bps: config.spread_bps as f64 / 10000.0,
            order_size: config.order_size_usd,
            skew_enabled: config.skew_inventory,
            min_spread_bps: config.min_spread_bps as f64 / 10000.0,
            max_spread_bps: config.max_spread_bps as f64 / 10000.0,
        }
    }

    pub fn calculate_quotes(&self, mid_price: f64) -> (f64, f64) {
        if mid_price <= 0.0 {
            return (0.0, 0.0);
        }

        let mut effective_spread = self.spread_bps;
        effective_spread = effective_spread.clamp(self.min_spread_bps, self.max_spread_bps);

        let half_spread = mid_price * effective_spread / 2.0;
        
        let mut bid = mid_price - half_spread;
        let mut ask = mid_price + half_spread;

        if self.skew_enabled {
            let position = self.get_position_signal();
            if position > 0.0 {
                ask -= half_spread * 0.2;
            } else if position < 0.0 {
                bid += half_spread * 0.2;
            }
        }

        bid = bid.clamp(0.01, 0.99);
        ask = ask.clamp(0.01, 0.99);
        
        if bid >= ask {
            let mid = (bid + ask) / 2.0;
            let half = (ask - bid).abs() / 2.0 + 0.01;
            bid = (mid - half).clamp(0.01, 0.99);
            ask = (mid + half).clamp(0.01, 0.99);
        }

        (bid, ask)
    }

    fn get_position_signal(&self) -> f64 {
        0.0
    }

    pub fn order_size(&self) -> f64 {
        self.order_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_quoter(spread_bps: u32) -> Quoter {
        let config = StrategyConfig {
            market_ids: vec![],
            spread_bps,
            order_size_usd: 1000.0,
            refresh_interval_ms: 100,
            skew_inventory: false,
            min_spread_bps: 50,
            max_spread_bps: 500,
        };
        Quoter::new(&config)
    }

    #[test]
    fn test_calculate_quotes_basic() {
        let quoter = create_test_quoter(100);
        let (bid, ask) = quoter.calculate_quotes(0.50);
        
        assert!(bid < ask, "Bid should be less than ask");
        assert!(bid >= 0.01 && bid <= 0.99, "Bid should be in valid range");
        assert!(ask >= 0.01 && ask <= 0.99, "Ask should be in valid range");
    }

    #[test]
    fn test_calculate_quotes_zero_price() {
        let quoter = create_test_quoter(100);
        let (bid, ask) = quoter.calculate_quotes(0.0);
        
        assert_eq!(bid, 0.0);
        assert_eq!(ask, 0.0);
    }

    #[test]
    fn test_quotes_within_range() {
        let quoter = create_test_quoter(100);
        let (bid, ask) = quoter.calculate_quotes(0.99);
        
        assert!(bid <= 0.99);
        assert!(ask <= 0.99);
    }

    #[test]
    fn test_spread_calculation() {
        let quoter = create_test_quoter(100);
        let (bid, ask) = quoter.calculate_quotes(0.50);
        
        let spread = ask - bid;
        assert!(spread > 0.0, "Spread should be positive");
        assert!(spread >= 0.0025, "Spread should be at least 0.5%");
    }

    #[test]
    fn test_min_max_spread() {
        let config = StrategyConfig {
            market_ids: vec![],
            spread_bps: 100,
            order_size_usd: 1000.0,
            refresh_interval_ms: 100,
            skew_inventory: false,
            min_spread_bps: 50,
            max_spread_bps: 200,
        };
        let quoter = Quoter::new(&config);
        
        let (bid, ask) = quoter.calculate_quotes(0.50);
        let spread_bps = ((ask - bid) / ((bid + ask) / 2.0)) * 10000.0;
        
        assert!(spread_bps >= 50.0, "Should enforce min spread");
        assert!(spread_bps <= 200.0, "Should enforce max spread");
    }
}
