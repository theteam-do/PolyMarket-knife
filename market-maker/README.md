# Market Maker - 返佣做市策略

## 🎯 策略核心

在 Polymarket 双边挂单提供流动性，赚取：
1. **返佣** - 被 Taker 成交时获得 15%~25% 手续费返还
2. **价差** - Bid/Ask 之间的 spread

## 📈 收益模型

```
日收益 = 成交量 × (返佣率 + 价差率) - Gas 成本

典型参数:
- 日成交量: $100,000
- 返佣率: 0.2%
- 价差率: 0.1%
- 日收益: $300
- 年化: ~$100,000 (取决于资金利用率)
```

## ⚡ 性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 撤单延迟 | <50ms | 从信号到撤单请求发出 |
| 下单延迟 | <30ms | 从决策到订单发出 |
| 心跳间隔 | 100ms | 订单健康检查 |
| 最大持仓 | 可配置 | 默认 $10,000 |

## 🔧 配置示例

```toml
# config/market-maker.toml
[polygong]
rpc_url = "wss://polygon-rpc.com"
private_key = "0x..."  #  NEVER commit this

[exchange]
api_key = "..."
api_secret = "..."

[strategy]
markets = ["0x...", "0x..."]  # 市场合约地址
spread_bps = 100              # 1% 价差 (100 = 1%)
max_position_usd = 10000      # 最大持仓
order_size_usd = 1000         # 单笔订单大小
refresh_interval_ms = 100     # 刷新间隔

[risk]
max_loss_per_day = 500        # 日最大亏损
stop_loss_pct = 5             # 止损百分比
```

## 🏗️ 架构设计

```
┌─────────────────────────────────────────────────────┐
│                    Event Loop                        │
│  (单线程，无锁，CPU 绑定)                             │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐      │
│  │  WS      │───▶│  Order   │───▶│  Risk    │      │
│  │  Feed    │    │  Book    │    │  Manager │      │
│  └──────────┘    └──────────┘    └──────────┘      │
│       │               │               │             │
│       ▼               ▼               ▼             │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐      │
│  │  Price   │    │  Quote   │    │  Order   │      │
│  │  Update  │    │  Engine  │    │  Executor│      │
│  └──────────┘    └──────────┘    └──────────┘      │
│                                                      │
└─────────────────────────────────────────────────────┘
```

## 📝 核心逻辑

```rust
// 1. 接收订单簿更新
fn on_orderbook_update(&mut self, update: OrderBookUpdate) {
    self.order_book.apply(update);
    self.check_requote();
}

// 2. 检查是否需要重新报价
fn check_requote(&mut self) {
    if self.needs_requote() {
        self.cancel_all_orders();
        self.place_new_quotes();
    }
}

// 3. 计算最优报价
fn calculate_quotes(&self) -> (Price, Price) {
    let mid = self.order_book.mid_price();
    let spread = self.config.spread_bps as f64 / 10000.0;
    let bid = mid * (1.0 - spread / 2.0);
    let ask = mid * (1.0 + spread / 2.0);
    (bid, ask)
}

// 4. 风控检查
fn check_risk(&self, side: Side, size: f64) -> bool {
    let current_position = self.get_position();
    let new_position = current_position + size * side.sign();
    new_position.abs() <= self.config.max_position_usd
}
```

## 🚨 风控规则

1. **持仓限制** - 单市场最大持仓不超过配置值
2. **日亏损限制** - 达到日亏损上限自动停止
3. **库存偏斜** - 持仓过大时调整报价偏向平仓
4. **波动率保护** - 高波动时扩大价差或暂停

## 🔍 监控指标

```bash
# Prometheus metrics
market_maker_orders_active      # 活跃订单数
market_maker_position_usd       # 当前持仓
market_maker_pnl_usd            # 累计盈亏
market_maker_rebate_usd         # 累计返佣
market_maker_latency_ms         # 订单延迟
```

## 🛠️ 部署建议

```bash
# 1. 使用低延迟 VPS (推荐 AWS us-east-1)
# 2. CPU 隔离和绑定
isolcpus=2,3
taskset -c 2 ./target/release/market-maker

# 3. 网络优化
ethtool -K eth0 gro off
ethtool -K eth0 gso off

# 4. 监控
prometheus --config.file=prometheus.yml
grafana-server
```
