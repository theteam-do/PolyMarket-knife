//! Polymarket 官方 SDK 适配层
//! 
//! 本模块提供对官方 `polymarket-client-sdk` 的封装，简化策略代码迁移。

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
pub use polymarket_client_sdk::clob::types::{Side, OrderType};
