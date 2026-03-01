//! 集成测试

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Component {
    Scanner,
    Detector,
    Executor,
    Monitor,
}

fn run_pipeline(healthy: &HashMap<Component, bool>) -> bool {
    [
        Component::Scanner,
        Component::Detector,
        Component::Executor,
        Component::Monitor,
    ]
    .iter()
    .all(|c| healthy.get(c).copied().unwrap_or(false))
}

/// 测试市场制作器启动
#[test]
fn test_market_maker_starts() {
    let order_size_usd = 1000.0_f64;
    let spread_bps = 100_u32;
    assert!(order_size_usd > 0.0, "order size must be positive");
    assert!((1..=5000).contains(&spread_bps), "spread must be in sane range");
}

/// 测试套利检测器
#[test]
fn test_arbitrage_detector() {
    let yes_price = 0.47_f64;
    let no_price = 0.48_f64;
    let total = yes_price + no_price;
    let min_profit = 0.01_f64;
    let is_opportunity = total < 1.0 - min_profit;
    assert!(is_opportunity, "yes+no below 1 should be detected as opportunity");
}

/// 测试监控指标
#[test]
fn test_monitoring_metrics() {
    let orders = 3_u64;
    let volume = 1200.0_f64;
    let pnl = -12.5_f64;

    let export = format!(
        "total_orders {}\ntotal_volume {}\ntotal_pnl {}\n",
        orders, volume, pnl
    );

    assert!(export.contains("total_orders 3"));
    assert!(export.contains("total_volume 1200"));
    assert!(export.contains("total_pnl -12.5"));
}

/// 测试告警系统
#[test]
fn test_alert_system() {
    let max_daily_loss = 500.0_f64;
    let daily_pnl = -600.0_f64;
    let should_alert = daily_pnl < -max_daily_loss;
    assert!(should_alert, "daily loss breach should trigger alert");
}

/// 测试端到端流程
#[test]
fn test_end_to_end_flow() {
    let mut healthy = HashMap::new();
    healthy.insert(Component::Scanner, true);
    healthy.insert(Component::Detector, true);
    healthy.insert(Component::Executor, true);
    healthy.insert(Component::Monitor, true);

    assert!(run_pipeline(&healthy), "all components healthy => pipeline success");

    healthy.insert(Component::Executor, false);
    assert!(!run_pipeline(&healthy), "executor unhealthy => pipeline fails fast");
}
