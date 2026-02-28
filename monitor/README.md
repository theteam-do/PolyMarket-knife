# 监控告警模块

## 监控指标

### PnL 指标
- `daily_pnl_usd` - 日盈亏
- `total_pnl_usd` - 总盈亏
- `max_drawdown_usd` - 最大回撤

### 订单指标
- `orders_placed_total` - 下单总数
- `orders_filled_total` - 成交总数
- `orders_cancelled_total` - 取消总数
- `orders_failed_total` - 失败总数

### 业务指标
- `opportunities_found_total` - 机会发现数
- `opportunities_executed_total` - 机会执行数
- `signals_generated_total` - 信号生成数

### 性能指标
- `api_latency_ms` - API 延迟 (直方图)
- `order_latency_ms` - 下单延迟 (直方图)

### 风控指标
- `consecutive_losses` - 连续亏损次数
- `risk_exposure_usd` - 风险敞口
- `total_position_usd` - 总持仓
- `position_per_market_usd` - 单市场持仓

## 告警规则

| 告警类型 | 触发条件 | 级别 |
|----------|----------|------|
| 日亏损超限 | PnL < -$500 | Critical |
| 日亏损警告 | PnL < -$400 (80%) | Warning |
| 持仓超限 | 持仓 > $10,000 | Critical |
| 延迟过高 | 延迟 > 100ms | Warning |
| 连续亏损 | 连续 5 次亏损 | Critical |
| API 错误率 | 错误率 > 10% | Warning |

## 使用示例

```rust
use monitor::{Metrics, AlertManager, AlertConfig, Dashboard};

// 创建监控
let metrics = Metrics::new();
let alerts = AlertManager::new(AlertConfig::default());
let dashboard = Dashboard::new(metrics.clone(), alerts);

// 记录订单
let timer = Timer::new();
// ... 下单逻辑 ...
metrics.record_order(OrderStatus::Filled, timer.elapsed_ms());

// 记录 PnL
metrics.record_pnl(dec!(100));

// 检查告警
let pnl = dec!(-450);
let alert_list = alerts.check_daily_loss(pnl);
if alerts.should_stop_trading() {
    println!("停止交易！");
}

// 打印状态
dashboard.print_status();

// 导出 Prometheus 指标
let metrics_output = metrics.gather();
println!("{}", metrics_output);
```

## Prometheus 集成

启动 HTTP 服务器:

```bash
#  metrics server 监听 9090 端口
./target/release/monitor-server --port 9090
```

Prometheus 配置:

```yaml
scrape_configs:
  - job_name: 'polymarket'
    static_configs:
      - targets: ['localhost:9090']
```

Grafana 仪表板 ID: `TODO`

## 告警通知

支持以下通知方式:
- [ ] 邮件通知
- [ ] Slack webhook
- [ ] Telegram bot
- [ ] 钉钉 webhook

