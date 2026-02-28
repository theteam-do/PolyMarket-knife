//! 报价引擎

use crate::config::StrategyConfig;
use crate::order_book::MarketOrderBook;
use rust_decimal::Decimal;

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

    pub fn calculate_quotes(&self, book: &MarketOrderBook) -> (f64, f64) {
        let Some(mid) = book.mid_price() else {
            return (0.0, 0.0);
        };

        if mid <= 0.0 {
            return (0.0, 0.0);
        }

        // 基础价差
        let mut effective_spread = self.spread_bps;

        // 根据市场波动率调整价差
        if let Some(spread_bps) = book.spread_bps() {
            let market_spread = spread_bps as f64 / 10000.0;
            effective_spread = (market_spread * 1.2).max(self.min_spread_bps);
        }

        // 限制在最小/最大范围内
        effective_spread = effective_spread.clamp(self.min_spread_bps, self.max_spread_bps);

        let half_spread = mid * effective_spread / 2.0;

        let mut bid = mid - half_spread;
        let mut ask = mid + half_spread;

        // 库存偏斜
        if self.skew_enabled {
            let position = self.get_position_signal();
            if position > 0.0 {
                ask -= half_spread * 0.2;
            } else if position < 0.0 {
                bid += half_spread * 0.2;
            }
        }

        // 确保价格在合理范围内
        bid = bid.clamp(0.01, 0.99);
        ask = ask.clamp(0.01, 0.99);

        // 确保 bid < ask
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
    use crate::order_book::{Level, MarketOrderBook};

    fn create_test_quoter() -> Quoter {
        let config = StrategyConfig {
            market_ids: vec![],
            spread_bps: 100,
            order_size_usd: 1000.0,
            refresh_interval_ms: 100,
            skew_inventory: false,
            min_spread_bps: 50,
            max_spread_bps: 500,
        };
        Quoter::new(&config)
    }

    fn create_test_book(bid: f64, ask: f64) -> MarketOrderBook {
        let mut book = MarketOrderBook::new("test".to_string());
        book.bids = vec![Level {
            price: bid,
            size: 100.0,
        }];
        book.asks = vec![Level {
            price: ask,
            size: 100.0,
        }];
        book.best_bid = Some(bid);
        book.best_ask = Some(ask);
        book
    }

    #[test]
    fn test_calculate_quotes_basic() {
        let quoter = create_test_quoter();
        let book = create_test_book(0.50, 0.52);

        let (bid, ask) = quoter.calculate_quotes(&book);

        assert!(bid < ask, "Bid should be less than ask");
        assert!(bid >= 0.01 && bid <= 0.99, "Bid should be in valid range");
        assert!(ask >= 0.01 && ask <= 0.99, "Ask should be in valid range");
    }

    #[test]
    fn test_calculate_quotes_empty_book() {
        let quoter = create_test_quoter();
        let book = MarketOrderBook::new("test".to_string());

        let (bid, ask) = quoter.calculate_quotes(&book);

        assert_eq!(bid, 0.0);
        assert_eq!(ask, 0.0);
    }

    #[test]
    fn test_quotes_within_range() {
        let quoter = create_test_quoter();
        let book = create_test_book(0.99, 1.00);

        let (bid, ask) = quoter.calculate_quotes(&book);

        assert!(bid <= 0.99);
        assert!(ask <= 0.99);
    }

    #[test]
    fn test_quotes_spread_calculation() {
        let quoter = create_test_quoter();
        let book = create_test_book(0.50, 0.51);

        let (bid, ask) = quoter.calculate_quotes(&book);

        let spread = ask - bid;
        assert!(spread > 0.0, "Spread should be positive");
    }
}
