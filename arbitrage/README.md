# Arbitrage - 套利策略

## 🚦 当前实现状态（2026-03）

- 当前为**框架 + 执行意图提交**版本，不包含完整链上 `mint/redeem` 交互。
- 支持 `execution.mode = "paper|live"`：
  - `paper`：仅模拟收益
  - `live`：提交执行意图到配置的执行端点
- 默认启用安全门禁：`require_explicit_live_ack = true` 且 `live_acknowledged = false`。

## 🎯 策略核心

捕捉 Polymarket 内部的定价错误，无风险获利。

## 📊 套利类型

### 1. 铸造 - 赎回套利
当 `Yes 价格 + No 价格 ≠ $1` 时：
- 如果 `Yes + No < $1`：买入 Yes+No，铸造后赎回获利
- 如果 `Yes + No > $1`：借入 Yes+No，赎回后卖出获利

### 2. 跨市场价差
同一事件在不同市场的价格差异（较少见）

## ⚡ 性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 扫描延迟 | 视部署环境 | 全市场扫描周期 |
| 执行延迟 | 视执行端点 | 发现机会到提交执行意图 |
| 最小利润 | $0.02 | 扣除 Gas 后的净利润 |

## 🔧 配置示例

```toml
# config/arbitrage.toml
[polygon]
rpc_url = "wss://polygon-bor-rpc.publicnode.com"
private_key = "0x..."

[clob]
host = "https://clob.polymarket.com"
api_key = ""
api_secret = ""

[strategy]
min_profit_usd = 0.02
max_position_per_trade = 1000
scan_interval_ms = 50
gas_price_gwei = 50
include_all = true
exclude_market_ids = ["0x...", "0x..."]

[execution]
mode = "paper"                  # paper 或 live
environment = "testnet"         # testnet 或 mainnet
require_explicit_live_ack = true
live_acknowledged = false
live_failure_fallback_to_paper = false
```

## 🏗️ 架构设计

```
┌────────────────────────────────────────┐
│           Scanner (单线程)              │
│                                        │
│  ┌─────────┐  ┌─────────┐  ┌────────┐ │
│  │ Market  │  │ Market  │  │ Market │ │
│  │  1      │  │  2      │  │  N     │ │
│  └────┬────┘  └────┬────┘  └───┬────┘ │
│       │           │           │       │
│       └───────────┼───────────┘       │
│                   ▼                   │
│           ┌───────────────┐           │
│           │  Arbitrage    │           │
│           │  Detector     │           │
│           └───────┬───────┘           │
│                   │                   │
│                   ▼                   │
│           ┌───────────────┐           │
│           │  Executor     │           │
│           └───────────────┘           │
└────────────────────────────────────────┘
```

## 📝 核心逻辑

```rust
// 1. 扫描所有市场
fn scan_markets(&self) -> Vec<MarketPrice> {
    self.markets.iter()
        .filter_map(|m| self.fetch_price(m))
        .collect()
}

// 2. 检测套利机会
fn detect_arbitrage(&self, prices: &[MarketPrice]) -> Option<ArbOpportunity> {
    for market in prices {
        let sum = market.yes_price + market.no_price;
        
        if sum < 1.0 - self.min_profit {
            // 买入套利机会
            return Some(ArbOpportunity::BuyAndMint {
                profit: 1.0 - sum,
                market: market.address,
            });
        }
        
        if sum > 1.0 + self.min_profit {
            // 卖出套利机会
            return Some(ArbOpportunity::RedeemAndSell {
                profit: sum - 1.0,
                market: market.address,
            });
        }
    }
    None
}

// 3. 执行套利
async fn execute(&self, opp: &ArbOpportunity) -> Result<()> {
    match opp {
        ArbOpportunity::BuyAndMint { market, .. } => {
            // 当前实现提交执行意图，由执行端处理
            self.submit_execution_intent(market).await?;
        }
        ArbOpportunity::RedeemAndSell { market, .. } => {
            // 反向操作
        }
    }
    Ok(())
}
```

## ⚠️ 注意事项

1. **Gas 成本** - 小额套利可能被 Gas 吃掉利润
2. **成交风险** - 机会可能瞬间消失
3. **智能合约风险** - 铸造/赎回可能有额外限制

## 📈 预期收益

```
日扫描次数: 17,000 次 (50ms 间隔)
日机会数: 5-20 次
单次利润: $0.02-$0.50
日收益: $0.50-$5.00 (取决于资金规模)
```

套利机会较少，但风险极低，适合保守型策略。
