//! 共享配置类型
//!
//! 包含所有策略共享的配置结构，避免代码重复

use anyhow::Result;
use secrecy::SecretString;
use serde::Deserialize;

/// 执行模式
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// 模拟模式，不实际下单
    #[default]
    Paper,
    /// 实盘模式，实际下单
    Live,
}

/// 运行环境
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeEnvironment {
    /// 测试网环境
    #[default]
    Testnet,
    /// 主网环境
    Mainnet,
}

/// 执行配置
///
/// 控制策略的执行行为，包括模式、环境和安全确认
#[derive(Debug, Deserialize, Clone)]
pub struct ExecutionConfig {
    /// 执行模式（模拟/实盘）
    #[serde(default)]
    pub mode: ExecutionMode,
    /// 运行环境（测试网/主网）
    #[serde(default)]
    pub environment: RuntimeEnvironment,
    /// 是否需要显式的实盘确认
    #[serde(default = "default_true")]
    pub require_explicit_live_ack: bool,
    /// 是否已确认实盘风险
    #[serde(default)]
    pub live_acknowledged: bool,
    /// 实盘失败是否降级到模拟模式
    #[serde(default)]
    pub live_failure_fallback_to_paper: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Paper,
            environment: RuntimeEnvironment::Testnet,
            require_explicit_live_ack: true,
            live_acknowledged: false,
            live_failure_fallback_to_paper: false,
        }
    }
}

fn default_true() -> bool {
    true
}

impl ExecutionConfig {
    /// 验证执行配置的安全性
    ///
    /// # Errors
    /// 如果配置为实盘模式但未确认风险，返回错误
    pub fn enforce_safety(&self) -> Result<()> {
        if self.mode == ExecutionMode::Live
            && self.require_explicit_live_ack
            && !self.live_acknowledged
        {
            anyhow::bail!(
                "live mode requires explicit acknowledgement: set [execution].live_acknowledged = true"
            );
        }
        Ok(())
    }
}

/// Polygon 配置
#[derive(Debug, Deserialize, Clone)]
pub struct PolygonConfig {
    /// RPC URL
    pub rpc_url: String,
    /// WebSocket RPC URL（可选）
    pub ws_rpc_url: Option<String>,
    /// 私钥（可从环境变量加载），内存敏感，零化释放
    #[serde(default)]
    pub private_key: SecretString,
}

/// CLOB 配置
#[derive(Debug, Deserialize, Clone)]
pub struct ClobConfig {
    /// API 主机地址
    pub host: String,
    /// 市场数据 WebSocket URL
    pub ws_market_url: Option<String>,
    /// 用户数据 WebSocket URL
    pub ws_user_url: Option<String>,
    /// API Key
    #[serde(default)]
    pub api_key: Option<String>,
    /// API Secret
    #[serde(default)]
    pub api_secret: Option<String>,
    /// API Passphrase
    #[serde(default)]
    pub passphrase: Option<String>,
    /// Proxy URL
    #[serde(default)]
    pub proxy_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.mode, ExecutionMode::Paper);
        assert_eq!(config.environment, RuntimeEnvironment::Testnet);
        assert!(config.require_explicit_live_ack);
        assert!(!config.live_acknowledged);
    }

    #[test]
    fn test_enforce_safety_passes_for_paper() {
        let config = ExecutionConfig::default();
        assert!(config.enforce_safety().is_ok());
    }

    #[test]
    fn test_enforce_safety_fails_for_unacknowledged_live() {
        let config = ExecutionConfig {
            mode: ExecutionMode::Live,
            require_explicit_live_ack: true,
            live_acknowledged: false,
            ..Default::default()
        };
        assert!(config.enforce_safety().is_err());
    }

    #[test]
    fn test_enforce_safety_passes_for_acknowledged_live() {
        let config = ExecutionConfig {
            mode: ExecutionMode::Live,
            live_acknowledged: true,
            ..Default::default()
        };
        assert!(config.enforce_safety().is_ok());
    }
}
