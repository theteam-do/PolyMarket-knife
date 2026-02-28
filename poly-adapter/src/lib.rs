//! Polymarket 官方 SDK 适配层
//! 
//! 本模块提供对官方 `polymarket-client-sdk` 的封装，简化策略代码迁移。
//! 
//! ## 主要功能
//! 
//! - **认证适配**: 简化的认证流程
//! - **类型转换**: 统一类型定义
//! - **错误转换**: 友好的错误类型
//! - **API 封装**: 简化官方 API 调用
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use poly_adapter::PolyAdapter;
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 创建适配器（会自动认证）
//!     let adapter = PolyAdapter::new(
//!         "https://clob.polymarket.com",
//!         "YOUR_PRIVATE_KEY"
//!     ).await?;
//!     
//!     // 获取订单簿
//!     let orderbook = adapter.get_orderbook("TOKEN_ID").await?;
//!     
//!     // 下单
//!     let order_id = adapter.buy("TOKEN_ID", 0.50, 100.0).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod adapter;
pub mod auth;
pub mod error;
pub mod types;

// 重新导出常用类型
pub use adapter::PolyAdapter;
pub use auth::AuthConfig;
pub use error::{Error, Result};
pub use types::*;

// 重新导出官方 SDK 类型
pub use polymarket_client_sdk::clob::types::{Side, OrderType, OrderStatus};
pub use polymarket_client_sdk::types::Address;
