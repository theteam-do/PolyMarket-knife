//! 报价引擎

use crate::config::StrategyConfig;

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

    pub fn calculate_quotes_with_position(
        &self,
        mid_price: f64,
        position_signal: f64,
    ) -> (f64, f64) {
        if mid_price <= 0.0 {
            return (0.0, 0.0);
        }

        let effective_spread = self
            .spread_bps
            .clamp(self.min_spread_bps, self.max_spread_bps);
        let half_spread = mid_price * effective_spread / 2.0;

        let mut bid = mid_price - half_spread;
        let mut ask = mid_price + half_spread;

        if self.skew_enabled {
            let skew = position_signal.clamp(-1.0, 1.0);
            let adjustment = half_spread * 0.2 * skew.abs();
            if skew > 0.0 {
                bid -= adjustment / 2.0;
                ask -= adjustment;
            } else if skew < 0.0 {
                bid += adjustment;
                ask += adjustment / 2.0;
            }
        }

        // Clamp within [0.001, 0.999] to allow edge quotes near 0/1
        bid = bid.clamp(0.001, 0.999);
        ask = ask.clamp(0.001, 0.999);

        if bid >= ask {
            // If clamping collapsed the spread, use minimum spread
            let mid = (bid + ask) / 2.0;
            let min_half = mid * (self.min_spread_bps / 2.0).max(0.0005);
            bid = (mid - min_half).clamp(0.001, 0.999);
            ask = (mid + min_half).clamp(0.001, 0.999);
        }

        (bid, ask)
    }

    pub fn order_size(&self) -> f64 {
        self.order_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SideMode;

    fn create_test_quoter(spread_bps: u32) -> Quoter {
        let config = StrategyConfig {
            market_ids: vec![],
            spread_bps,
            order_size_usd: 1000.0,
            refresh_interval_ms: 100,
            skew_inventory: false,
            min_spread_bps: 50,
            max_spread_bps: 500,
            side_mode: SideMode::TwoSided,
            metrics_bind_addr: "127.0.0.1:9090".to_string(),
        };
        Quoter::new(&config)
    }

    #[test]
    fn test_calculate_quotes_basic() {
        let quoter = create_test_quoter(100);
        let (bid, ask) = quoter.calculate_quotes_with_position(0.50, 0.0);

        assert!(bid < ask, "Bid should be less than ask");
        assert!(bid >= 0.001 && bid <= 0.999, "Bid should be in valid range");
        assert!(ask >= 0.001 && ask <= 0.999, "Ask should be in valid range");
    }

    #[test]
    fn test_calculate_quotes_zero_price() {
        let quoter = create_test_quoter(100);
        let (bid, ask) = quoter.calculate_quotes_with_position(0.0, 0.0);

        assert_eq!(bid, 0.0);
        assert_eq!(ask, 0.0);
    }

    #[test]
    fn test_quotes_within_range() {
        let quoter = create_test_quoter(100);
        let (bid, ask) = quoter.calculate_quotes_with_position(0.99, 0.0);

        assert!(bid < ask, "Bid should be less than ask");
        assert!(bid >= 0.001, "Bid should not exceed lower bound");
        assert!(ask <= 0.999, "Ask should not exceed upper bound");
    }

    #[test]
    fn test_spread_calculation() {
        let quoter = create_test_quoter(100);
        let (bid, ask) = quoter.calculate_quotes_with_position(0.50, 0.0);

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
            side_mode: SideMode::TwoSided,
            metrics_bind_addr: "127.0.0.1:9090".to_string(),
        };
        let quoter = Quoter::new(&config);

        let (bid, ask) = quoter.calculate_quotes_with_position(0.50, 0.0);
        let spread_bps = ((ask - bid) / ((bid + ask) / 2.0)) * 10000.0;

        assert!(spread_bps >= 50.0, "Should enforce min spread");
        assert!(spread_bps <= 200.0, "Should enforce max spread");
    }

    #[test]
    fn test_inventory_skew_changes_quotes() {
        let config = StrategyConfig {
            market_ids: vec![],
            spread_bps: 100,
            order_size_usd: 1000.0,
            refresh_interval_ms: 100,
            skew_inventory: true,
            min_spread_bps: 50,
            max_spread_bps: 200,
            side_mode: SideMode::TwoSided,
            metrics_bind_addr: "127.0.0.1:9090".to_string(),
        };
        let quoter = Quoter::new(&config);

        let (_plain_bid, plain_ask) = quoter.calculate_quotes_with_position(0.50, 0.0);
        let (_skewed_bid, skewed_ask) = quoter.calculate_quotes_with_position(0.50, 1.0);

        assert!(
            skewed_ask < plain_ask,
            "Long inventory should bias toward selling"
        );
    }
}
