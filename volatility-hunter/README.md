# Volatility Hunter - 超短期波动狩猎

## 🚦 当前实现状态（2026-03）

- 当前为**信号生成 + 执行框架**版本。
- 支持 `execution.mode = "paper|live"`：
  - `paper`：模拟执行
  - `live`：尝试真实下单
- 默认启用安全门禁：`require_explicit_live_ack = true` 且 `live_acknowledged = false`。
- 延迟目标为设计目标，当前版本仅具备基础日志可观测性。

## 🎯 策略核心

利用**毫秒级数据源**（币安 WebSocket）比 Polymarket 订单簿更快，捕捉加密货币波动瞬间的定价延迟。

## 📊 狩猎场景

### 场景 1: 低概率埋伏
- **时机**: 市场恐慌/贪婪极端时
- **操作**: 以 3¢-5¢买入被低估的选项
- **回报**: 33 倍 + (如果事件反转)
- **胜率**: 20%-30%

### 场景 2: 确定性重仓
- **时机**: 窗口最后 10 秒，趋势已明但价格未完全反应
- **操作**: 重仓吃 30%-50%涨幅
- **回报**: 单笔 30%-50%
- **胜率**: 70%-80%

## ⚡ 性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 数据源延迟 | 目标值 | 币安 WS 到本地处理 |
| 决策延迟 | 目标值 | 信号生成到下单决策 |
| 下单延迟 | 目标值 | 决策到订单发出 |
| 总延迟 | 目标值 | 端到端延迟 |

## 🔧 配置示例

```toml
# config/volatility-hunter.toml
[polygon]
rpc_url = "wss://polygon-bor-rpc.publicnode.com"
private_key = "0x..."

[clob]
host = "https://clob.polymarket.com"
api_key = ""
api_secret = ""

[binance]
ws_url = "wss://stream.binance.com:9443/ws"
api_key = "..."
api_secret = "..."

[strategy]
# 监控的交易对
symbols = ["BTCUSDT", "ETHUSDT"]

# 波动率阈值
volatility_threshold = 0.02      # 2% 波动触发
momentum_threshold = 0.01        # 1% 动量触发

# 仓位管理
base_position_usd = 100          # 基础仓位 (埋伏用)
max_position_usd = 10000         # 最大仓位 (确定性用)
confidence_high = 0.8            # 高置信度阈值

# 风控
max_loss_per_trade = 100
max_daily_loss = 500
stop_loss_pct = 0.1              # 10% 止损

[execution]
mode = "paper"                  # paper 或 live
environment = "testnet"         # testnet 或 mainnet
require_explicit_live_ack = true
live_acknowledged = false
live_failure_fallback_to_paper = false
```

## 🏗️ 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    Event Loop (CPU Core 0)                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐          ┌──────────────┐                │
│  │  Binance WS  │─────────▶│   Signal     │                │
│  │  (Thread 1)  │  <2ms    │   Generator  │                │
│  └──────────────┘          └──────┬───────┘                │
│                                   │                         │
│                                   ▼                         │
│                          ┌──────────────┐                   │
│                          │  Confidence  │                   │
│                          │  Calculator  │                   │
│                          └──────┬───────┘                   │
│                                 │                           │
│                    ┌────────────┼────────────┐              │
│                    │                         │              │
│                    ▼                         ▼              │
│           ┌──────────────┐          ┌──────────────┐       │
│           │  Low Conf    │          │  High Conf   │       │
│           │  (小仓位)     │          │  (大仓位)     │       │
│           └──────┬───────┘          └──────┬───────┘       │
│                  │                         │                │
│                  └───────────┬─────────────┘                │
│                              ▼                              │
│                    ┌──────────────┐                         │
│                    │  Poly WS     │                         │
│                    │  Executor    │                         │
│                    └──────────────┘                         │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 📝 核心逻辑

```rust
// 1. 币安价格更新处理
fn on_binance_tick(&mut self, symbol: &str, price: f64) {
    let start = Instant::now();
    
    // 计算波动率
    let volatility = self.calc_volatility(symbol, price);
    
    // 计算动量
    let momentum = self.calc_momentum(symbol, price);
    
    // 生成信号
    if volatility > self.config.volatility_threshold {
        let signal = if momentum > 0.0 {
            Signal::Buy { confidence: self.calc_confidence(volatility, momentum) }
        } else {
            Signal::Sell { confidence: self.calc_confidence(volatility, momentum) }
        };
        
        // 执行信号
        self.execute_signal(signal);
    }
    
    // 记录延迟
    self.latency_histogram.record(start.elapsed().as_millis());
}

// 2. 置信度计算
fn calc_confidence(&self, volatility: f64, momentum: f64) -> f64 {
    let mut confidence = 0.5;
    
    // 波动率越高，置信度越高
    confidence += (volatility / 0.05).min(0.3);
    
    // 动量越强，置信度越高
    confidence += (momentum.abs() / 0.02).min(0.2);
    
    // 时间窗口因素 (最后 10 秒置信度提升)
    if self.time_to_expiry() < 10.0 {
        confidence += 0.1;
    }
    
    confidence.min(1.0)
}

// 3. 仓位管理
fn calculate_position(&self, confidence: f64) -> f64 {
    if confidence >= self.config.confidence_high {
        // 高置信度：大仓位
        self.config.max_position_usd
    } else {
        // 低置信度：小仓位埋伏
        self.config.base_position_usd
    }
}

// 4. 执行
async fn execute_signal(&mut self, signal: Signal) {
    let position = self.calculate_position(signal.confidence());
    
    // 快速下单
    self.executor.place_order(signal.market(), signal.side(), position).await;
}
```

## 🎯 信号类型

| 信号 | 触发条件 | 仓位 | 预期收益 |
|------|----------|------|----------|
| 埋伏 | 波动率>2%, 动量>1% | $100 | 33 倍 (20% 胜率) |
| 趋势 | 波动率>3%, 动量>2% | $1,000 | 50% (60% 胜率) |
| 确定性 | 波动率>5%, 动量>3%, 时间<10s | $10,000 | 30% (80% 胜率) |

## ⚠️ 风险提示

1. **反向波动** - 判断错误可能瞬间被击穿
2. **技术门槛** - 数据源必须比 Polymarket 快
3. **服务器要求** - 需要<2ms 延迟的 VPS

## 📈 预期收益

```
典型日交易:
- 埋伏交易: 20 次，亏损 16 次 ($1,600)，盈利 4 次 ($5,200)
- 趋势交易: 5 次，盈利 3 次 ($1,500)，亏损 2 次 ($400)
- 确定性交易: 2 次，盈利 2 次 ($6,000)

日收益: ~$10,700
月收益: ~$200,000 (复利)
```

这是收益最高的可程序化策略，但技术要求也最高。
