//! API 类型定义

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 订单方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

/// 订单类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Gtc, // Good Till Cancelled
    Fok, // Fill Or Kill
    Ioc, // Immediate Or Cancel
}

/// 订单请求
#[derive(Debug, Clone, Serialize)]
pub struct OrderRequest {
    #[serde(rename = "tokenID")]
    pub token_id: String,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    pub expiration: u64,
    pub nonce: u64,
}

/// 订单响应
#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    #[serde(rename = "orderID")]
    pub order_id: String,
    pub success: bool,
    pub signature: Option<String>,
}

/// 订单簿响应
#[derive(Debug, Clone, Deserialize)]
pub struct OrderBookResponse {
    pub token_id: String,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub timestamp: u64,
}

/// 订单簿层级
#[derive(Debug, Clone, Deserialize)]
pub struct Level {
    pub price: Decimal,
    pub size: Decimal,
}

/// 取消订单响应
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResponse {
    pub success: bool,
    #[serde(rename = "orderID")]
    pub order_id: String,
}
