//! 公共类型定义

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// 订单簿层级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub size: Decimal,
}

/// 订单簿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub token_id: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: u64,
}

impl OrderBook {
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.first().map(|l| l.price)
    }

    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.first().map(|l| l.price)
    }

    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::from(2)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_best_bid() {
        let ob = OrderBook {
            token_id: "test".to_string(),
            bids: vec![OrderBookLevel {
                price: Decimal::from_f64_retain(0.50).unwrap(),
                size: Decimal::from(100),
            }],
            asks: vec![OrderBookLevel {
                price: Decimal::from_f64_retain(0.52).unwrap(),
                size: Decimal::from(100),
            }],
            timestamp: 1234567890,
        };

        assert_eq!(ob.best_bid(), Some(Decimal::from_f64_retain(0.50).unwrap()));
    }

    #[test]
    fn test_orderbook_mid_price() {
        let ob = OrderBook {
            token_id: "test".to_string(),
            bids: vec![OrderBookLevel {
                price: Decimal::from_f64_retain(0.50).unwrap(),
                size: Decimal::from(100),
            }],
            asks: vec![OrderBookLevel {
                price: Decimal::from_f64_retain(0.52).unwrap(),
                size: Decimal::from(100),
            }],
            timestamp: 1234567890,
        };

        assert_eq!(
            ob.mid_price(),
            Some(Decimal::from_f64_retain(0.51).unwrap())
        );
    }

    #[test]
    fn test_orderbook_empty() {
        let ob = OrderBook {
            token_id: "test".to_string(),
            bids: vec![],
            asks: vec![],
            timestamp: 1234567890,
        };

        assert_eq!(ob.best_bid(), None);
        assert_eq!(ob.mid_price(), None);
    }
}
