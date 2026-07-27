//! Common - 共享配置和工具类型
//!
//! 本 crate 包含所有 PolyMarket Knife 策略共享的配置类型和工具函数。
//!
//! # 主要功能
//!
//! - **执行配置**: `ExecutionConfig`, `ExecutionMode`, `RuntimeEnvironment`
//! - **安全验证**: 实盘模式风险确认机制
//!
//! # 使用示例
//!
//! ```rust
//! use common::{ExecutionConfig, ExecutionMode, RuntimeEnvironment};
//!
//! // 创建默认配置（模拟模式）
//! let config = ExecutionConfig::default();
//! assert_eq!(config.mode, ExecutionMode::Paper);
//!
//! // 验证配置安全性
//! assert!(config.enforce_safety().is_ok());
//!
//! // 实盘模式需要确认
//! let live_config = ExecutionConfig {
//!     mode: ExecutionMode::Live,
//!     live_acknowledged: true,
//!     ..Default::default()
//! };
//! assert!(live_config.enforce_safety().is_ok());
//! ```

pub mod config;
pub mod edge;
pub mod probability;
pub mod sizing;
pub mod telemetry;

pub use config::*;
pub use edge::*;
pub use probability::*;
pub use sizing::*;
pub use telemetry::*;
