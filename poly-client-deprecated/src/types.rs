//! 公共类型定义

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Gtc,
    Fok,
    Ioc,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
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

    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
}

/// 订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub token_id: String,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    pub expiration: u64,
}

/// 订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    #[serde(rename = "orderID")]
    pub order_id: String,
    pub status: String,
    pub signature: Option<String>,
}

/// 取消订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    #[serde(rename = "orderID")]
    pub order_id: String,
}

/// 取消订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub success: bool,
    pub order_id: String,
}

/// 市场信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    #[serde(rename = "conditionID")]
    pub condition_id: String,
    pub question: String,
    pub outcome: String,
    #[serde(rename = "clobTokenIds")]
    pub token_ids: Vec<String>,
    #[serde(rename = "minimumTickSize")]
    pub min_tick_size: Decimal,
    #[serde(rename = "negRisk")]
    pub neg_risk: bool,
    pub volume_24h: Decimal,
}

/// 用户持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_id: String,
    pub balance: Decimal,
    pub total_cost: Decimal,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    #[serde(rename = "orderID")]
    pub order_id: String,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
    pub fee: Decimal,
    pub timestamp: u64,
}

/// API 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub code: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "API Error: {}", self.error)
    }
}

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_best_bid() {
        let ob = OrderBook {
            token_id: "test".to_string(),
            bids: vec![
                OrderBookLevel {
                    price: Decimal::from_f64_retain(0.50).unwrap(),
                    size: Decimal::from(100),
                },
                OrderBookLevel {
                    price: Decimal::from_f64_retain(0.49).unwrap(),
                    size: Decimal::from(200),
                },
            ],
            asks: vec![OrderBookLevel {
                price: Decimal::from_f64_retain(0.51).unwrap(),
                size: Decimal::from(100),
            }],
            timestamp: 1234567890,
        };

        assert_eq!(ob.best_bid(), Some(Decimal::from_f64_retain(0.50).unwrap()));
        assert_eq!(ob.best_ask(), Some(Decimal::from_f64_retain(0.51).unwrap()));
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
        assert_eq!(ob.best_ask(), None);
        assert_eq!(ob.mid_price(), None);
        assert_eq!(ob.spread(), None);
    }
}
