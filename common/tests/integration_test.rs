//! Common crate 集成测试
//!
//! 测试配置加载、验证和安全性功能

use common::{ExecutionConfig, ExecutionMode, RuntimeEnvironment};

/// 测试执行配置的默认行为
#[test]
fn test_execution_config_default() {
    let config = ExecutionConfig::default();

    assert_eq!(config.mode, ExecutionMode::Paper);
    assert_eq!(config.environment, RuntimeEnvironment::Testnet);
    assert!(config.require_explicit_live_ack);
    assert!(!config.live_acknowledged);
    assert!(!config.live_failure_fallback_to_paper);
}

/// 测试模拟模式总是安全的
#[test]
fn test_paper_mode_is_always_safe() {
    let config = ExecutionConfig {
        mode: ExecutionMode::Paper,
        live_acknowledged: false,
        ..Default::default()
    };

    assert!(config.enforce_safety().is_ok());
}

/// 测试实盘模式需要确认
#[test]
fn test_live_mode_requires_acknowledgement() {
    let config = ExecutionConfig {
        mode: ExecutionMode::Live,
        live_acknowledged: false,
        require_explicit_live_ack: true,
        ..Default::default()
    };

    let result = config.enforce_safety();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("live_acknowledged"));
}

/// 测试已确认的实盘模式是安全的
#[test]
fn test_acknowledged_live_mode_is_safe() {
    let config = ExecutionConfig {
        mode: ExecutionMode::Live,
        live_acknowledged: true,
        ..Default::default()
    };

    assert!(config.enforce_safety().is_ok());
}

/// 测试可以禁用显式确认要求
#[test]
fn test_can_disable_explicit_ack_requirement() {
    let config = ExecutionConfig {
        mode: ExecutionMode::Live,
        live_acknowledged: false,
        require_explicit_live_ack: false,
        ..Default::default()
    };

    assert!(config.enforce_safety().is_ok());
}

/// 测试环境配置独立于模式
#[test]
fn test_environment_is_independent_of_mode() {
    // 测试网 + 实盘模式
    let config = ExecutionConfig {
        mode: ExecutionMode::Live,
        environment: RuntimeEnvironment::Testnet,
        live_acknowledged: true,
        ..Default::default()
    };
    assert!(config.enforce_safety().is_ok());
    assert_eq!(config.environment, RuntimeEnvironment::Testnet);

    // 主网 + 模拟模式
    let config = ExecutionConfig {
        mode: ExecutionMode::Paper,
        environment: RuntimeEnvironment::Mainnet,
        ..Default::default()
    };
    assert!(config.enforce_safety().is_ok());
    assert_eq!(config.environment, RuntimeEnvironment::Mainnet);
}

/// 测试配置克隆
#[test]
fn test_config_clone() {
    let config1 = ExecutionConfig {
        mode: ExecutionMode::Live,
        live_acknowledged: true,
        live_failure_fallback_to_paper: true,
        ..Default::default()
    };

    let config2 = config1.clone();

    assert_eq!(config1.mode, config2.mode);
    assert_eq!(config1.live_acknowledged, config2.live_acknowledged);
    assert_eq!(
        config1.live_failure_fallback_to_paper,
        config2.live_failure_fallback_to_paper
    );
}

/// 测试 Debug 实现
#[test]
fn test_config_debug() {
    let config = ExecutionConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("ExecutionConfig"));
    assert!(debug_str.contains("Paper"));
}
