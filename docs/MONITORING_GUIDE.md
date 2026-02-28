# 监控告警指南

## 监控指标

### 1. PnL 指标
- daily_pnl_usd - 日盈亏
- total_pnl_usd - 总盈亏
- max_drawdown_usd - 最大回撤

### 2. 订单指标
- orders_placed_total - 下单总数
- orders_filled_total - 成交总数
- orders_cancelled_total - 取消总数
- orders_failed_total - 失败总数

### 3. 业务指标
- opportunities_found_total - 机会发现数
- opportunities_executed_total - 机会执行数
- signals_generated_total - 信号生成数

### 4. 性能指标
- api_latency_ms - API 延迟
- order_latency_ms - 下单延迟

### 5. 风控指标
- consecutive_losses - 连续亏损次数
- risk_exposure_usd - 风险敞口
- total_position_usd - 总持仓

## 告警规则

| 告警类型 | 触发条件 | 级别 |
|----------|----------|------|
| 日亏损超限 | PnL < -500 | Critical |
| 日亏损警告 | PnL < -400 | Warning |
| 持仓超限 | 持仓 > 10000 | Critical |
| 延迟过高 | 延迟 > 100ms | Warning |
| 连续亏损 | 连续 5 次亏损 | Critical |

## 使用示例

```rust
use monitor::{Metrics, AlertManager, Dashboard};

let metrics = Metrics::new();
let alerts = AlertManager::default();
let dashboard = Dashboard::new(metrics, alerts);

// 记录订单
metrics.orders_placed.inc();
metrics.orders_filled.inc();

// 检查告警
let alerts_list = alerts.check_daily_loss(pnl);
if alerts.should_stop_trading() {
    println!("停止交易");
}

// 打印状态
dashboard.print_status();
```

## Prometheus 集成

```yaml
scrape_configs:
  - job_name: polymarket
    static_configs:
      - targets: [localhost:9090]
```

