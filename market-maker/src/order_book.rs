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
            self.spread().map(|spread| ((spread / mid) * 10000.0) as u32)
        })
    }
}
