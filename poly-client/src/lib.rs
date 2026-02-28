//! Polymarket CLOB API 客户端
//! 
//! 提供完整的 API 对接：
//! - 认证（签名）
//! - 订单簿数据
//! - 下单/撤单
//! - 市场数据
//! - 用户持仓

pub mod client;
pub mod auth;
pub mod types;
pub mod market;
pub mod order;
pub mod ws;

pub use client::PolyClient;
pub use auth::AuthConfig;
pub use types::*;
