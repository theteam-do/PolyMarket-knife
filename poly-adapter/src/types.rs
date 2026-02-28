//! 类型定义和转换

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

/// Token ID (使用字符串，便于配置)
pub type TokenId = String;

/// 价格
pub type Price = Decimal;

/// 数量
pub type Size = Decimal;

/// 订单 ID
pub type OrderId = String;

/// 市场 ID (条件 ID)
pub type MarketId = String;

/// 将字符串转换为 Token ID (U256)
pub fn str_to_token_id(s: &str) -> Result<polymarket_client_sdk::types::U256, Error> {
    Ok(polymarket_client_sdk::types::U256::from_str(s)?)
}

/// 将 Token ID 转换为字符串
pub fn token_id_to_str(token_id: polymarket_client_sdk::types::U256) -> String {
    token_id.to_string()
}

/// 将 f64 转换为 Decimal
pub fn f64_to_decimal(f: f64) -> Result<Decimal, Error> {
    Decimal::from_f64_retain(f).ok_or_else(|| Error::Conversion {
        from: "f64".to_string(),
        to: "Decimal".to_string(),
        reason: "Invalid f64 value".to_string(),
    })
}

/// 将 Decimal 转换为 f64
pub fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

/// 订单簿层级
#[derive(Debug, Clone)]
pub struct OrderBookLevel {
    pub price: Price,
    pub size: Size,
}

/// 订单簿
#[derive(Debug, Clone)]
pub struct OrderBook {
    pub token_id: TokenId,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: u64,
}

impl OrderBook {
    /// 获取最佳买价
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.first().map(|l| l.price)
    }

    /// 获取最佳卖价
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.first().map(|l| l.price)
    }

    /// 获取中间价
    pub fn mid_price(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / dec!(2)),
            _ => None,
        }
    }

    /// 获取价差
    pub fn spread(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
}

/// 订单
#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: OrderId,
    pub token_id: TokenId,
    pub price: Price,
    pub size: Size,
    pub side: Side,
}

/// 持仓
#[derive(Debug, Clone)]
pub struct Position {
    pub token_id: TokenId,
    pub balance: Size,
}

/// 交易记录
#[derive(Debug, Clone)]
pub struct Trade {
    pub order_id: OrderId,
    pub token_id: TokenId,
    pub side: Side,
    pub price: Price,
    pub size: Size,
    pub fee: Price,
    pub timestamp: u64,
}

// 错误类型
use crate::error::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_token_id() {
        let result = str_to_token_id("123456").unwrap();
        assert_eq!(result.to_string(), "123456");
    }

    #[test]
    fn test_f64_to_decimal() {
        let result = f64_to_decimal(0.50).unwrap();
        assert_eq!(result, dec!(0.50));
    }

    #[test]
    fn test_orderbook_mid_price() {
        let ob = OrderBook {
            token_id: "123".to_string(),
            bids: vec![OrderBookLevel {
                price: dec!(0.50),
                size: dec!(100),
            }],
            asks: vec![OrderBookLevel {
                price: dec!(0.52),
                size: dec!(100),
            }],
            timestamp: 1234567890,
        };

        assert_eq!(ob.mid_price(), Some(dec!(0.51)));
        assert_eq!(ob.spread(), Some(dec!(0.02)));
    }
}
