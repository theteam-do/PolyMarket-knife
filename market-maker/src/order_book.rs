//! 轻量级订单簿 - 无锁设计

use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Level {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone)]
pub struct MarketOrderBook {
    pub market_id: String,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub last_update: u64,
}

impl MarketOrderBook {
    pub fn new(market_id: String) -> Self {
        Self {
            market_id,
            bids: Vec::with_capacity(20),
            asks: Vec::with_capacity(20),
            best_bid: None,
            best_ask: None,
            last_update: 0,
        }
    }

    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn spread_bps(&self) -> Option<u32> {
        self.mid_price()
            .and_then(|mid| self.spread().map(|spread| (spread / mid * 10000.0) as u32))
    }
}

#[derive(Debug, Default)]
pub struct OrderBook {
    markets: HashMap<String, MarketOrderBook>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            markets: HashMap::new(),
        }
    }

    pub fn update(&mut self, market_id: &str, levels: OrderBookLevels) {
        let book = self
            .markets
            .entry(market_id.to_string())
            .or_insert_with(|| MarketOrderBook::new(market_id.to_string()));

        book.bids = levels.bids;
        book.asks = levels.asks;
        book.best_bid = book.bids.first().map(|l| l.price);
        book.best_ask = book.asks.first().map(|l| l.price);
        book.last_update = timestamp_ms();
    }

    pub fn get_market(&self, market_id: &str) -> Option<&MarketOrderBook> {
        self.markets.get(market_id)
    }

    pub fn get_market_mut(&mut self, market_id: &str) -> Option<&mut MarketOrderBook> {
        self.markets.get_mut(market_id)
    }

    pub fn markets(&self) -> impl Iterator<Item = &MarketOrderBook> {
        self.markets.values()
    }
}

#[derive(Debug, Default)]
pub struct OrderBookLevels {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

fn timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
