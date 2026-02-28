//! 订单簿模块

use polymarket_client_sdk::types::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Level {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub token_id: String,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
}

impl OrderBook {
    pub fn new(token_id: String) -> Self {
        Self {
            token_id,
            bids: Vec::with_capacity(20),
            asks: Vec::with_capacity(20),
            best_bid: None,
            best_ask: None,
        }
    }

    pub fn update_best(&mut self) {
        self.best_bid = self.bids.first().map(|l| l.price);
        self.best_ask = self.asks.first().map(|l| l.price);
    }

    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    pub fn mid_price_decimal(&self) -> Option<Decimal> {
        self.mid_price().and_then(|p| Decimal::from_f64_retain(p))
    }

    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn spread_bps(&self) -> Option<u32> {
        self.mid_price().and_then(|mid| {
            self.spread()
                .map(|spread| ((spread / mid) * 10000.0) as u32)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_creation() {
        let level = Level {
            price: 0.50,
            size: 100.0,
        };
        assert_eq!(level.price, 0.50);
        assert_eq!(level.size, 100.0);
    }

    #[test]
    fn test_market_order_book_new() {
        let book = OrderBook::new("test".to_string());
        assert_eq!(book.token_id, "test");
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
        assert_eq!(book.best_bid, None);
        assert_eq!(book.best_ask, None);
    }

    #[test]
    fn test_mid_price() {
        let mut book = OrderBook::new("test".to_string());
        book.bids = vec![Level {
            price: 0.50,
            size: 100.0,
        }];
        book.asks = vec![Level {
            price: 0.52,
            size: 100.0,
        }];
        book.update_best();

        let mid = book.mid_price();
        assert!(mid.is_some());
        assert_eq!(mid.unwrap(), 0.51);
    }

    #[test]
    fn test_mid_price_empty_book() {
        let book = OrderBook::new("test".to_string());
        assert_eq!(book.mid_price(), None);
    }

    #[test]
    fn test_spread() {
        let mut book = OrderBook::new("test".to_string());
        book.bids = vec![Level {
            price: 0.50,
            size: 100.0,
        }];
        book.asks = vec![Level {
            price: 0.52,
            size: 100.0,
        }];
        book.update_best();

        let spread = book.spread();
        assert!(spread.is_some());
        assert!((spread.unwrap() - 0.02).abs() < 0.001);
    }

    #[test]
    fn test_spread_bps() {
        let mut book = OrderBook::new("test".to_string());
        book.bids = vec![Level {
            price: 0.50,
            size: 100.0,
        }];
        book.asks = vec![Level {
            price: 0.51,
            size: 100.0,
        }];
        book.update_best();

        let spread_bps = book.spread_bps();
        assert!(spread_bps.is_some());
        assert!(spread_bps.unwrap() > 0);
    }

    #[test]
    fn test_order_book_new() {
        let book = OrderBook::new("test".to_string());
        assert_eq!(book.token_id, "test");
        assert_eq!(book.bids.len(), 0);
        assert_eq!(book.asks.len(), 0);
    }

    #[test]
    fn test_update_best() {
        let mut book = OrderBook::new("test".to_string());
        book.bids = vec![
            Level {
                price: 0.50,
                size: 100.0,
            },
            Level {
                price: 0.49,
                size: 200.0,
            },
        ];
        book.asks = vec![
            Level {
                price: 0.52,
                size: 100.0,
            },
            Level {
                price: 0.53,
                size: 200.0,
            },
        ];
        book.update_best();

        assert_eq!(book.best_bid, Some(0.50));
        assert_eq!(book.best_ask, Some(0.52));
    }
}
