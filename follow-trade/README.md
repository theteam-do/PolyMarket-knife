# Follow Trade - 跟单策略

## 🚦 当前实现状态（2026-03）

- 当前为**跟单框架 + HTTP 执行**版本。
- 数据源当前主要来自 API 拉取，不是完整链上事件订阅。
- 支持 `execution.mode = "paper|live"`：
  - `paper`：仅模拟收益
  - `live`：尝试真实下单
- 默认启用安全门禁：`require_explicit_live_ack = true` 且 `live_acknowledged = false`。

## 🎯 策略核心

监控 Polymarket 上的"聪明钱"地址，自动复制它们的交易。

## 📊 聪明钱特征

通过历史数据分析识别：
- **胜率高** - 历史交易胜率 > 60%
- **收益稳定** - 夏普比率 > 1.5
- **信息优势** - 经常在重大事件前建仓
- **资金规模** - 单笔交易 > $10,000

### 知名聪明钱地址
- `0x...RN1` - 胜率最高的交易员
- `0x...Bidou28old` - 大额交易员
- `0x...` - 其他已验证地址

## ⚡ 性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 监控延迟 | 视部署环境 | API 拉取到本地处理 |
| 跟单延迟 | 视执行端点 | 检测到交易到执行完成 |
| 滑点容忍 | <2% | 超过则放弃跟单 |

## 🔧 配置示例

```toml
# config/follow-trade.toml
[polygon]
rpc_url = "wss://polygon-bor-rpc.publicnode.com"
private_key = "0x..."

[clob]
host = "https://clob.polymarket.com"
api_key = ""
api_secret = ""

[strategy]
# 监控的聪明钱地址
smart_addresses = [
    "0x...",  # RN1
    "0x...",  # Bidou28old
]

# 跟单参数
min_trade_size_usd = 1000    # 最小跟单金额
max_trade_size_usd = 5000    # 最大跟单金额
copy_ratio = 0.1             # 跟单比例 (10%)
slippage_tolerance = 0.02    # 滑点容忍度

# 风控
max_position_per_market = 10000
max_daily_loss = 1000
blacklist = []               # 黑名单市场

[execution]
mode = "paper"                  # paper 或 live
environment = "testnet"         # testnet 或 mainnet
require_explicit_live_ack = true
live_acknowledged = false
live_failure_fallback_to_paper = false
```

## 🏗️ 架构设计

```
┌─────────────────────────────────────────────────┐
│              Data Monitor                        │
│         (当前以 API 拉取为主)                      │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│              Trade Detector                      │
│    (过滤聪明钱交易，计算跟单量)                   │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│              Risk Checker                        │
│    (检查风控规则，计算滑点)                       │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│              Order Executor                      │
│         (执行跟单交易)                            │
└─────────────────────────────────────────────────┘
```

## 📝 核心逻辑

```rust
// 1. 监听链上交易事件
async fn monitor_chain(&self) -> Result<Vec<TradeEvent>> {
    let events = self.contract.events()
        .from_block(BlockNumber::Latest)
        .query()
        .await?;
    
    Ok(events.into_iter()
        .filter(|e| self.is_smart_money(e.from))
        .collect())
}

// 2. 计算跟单量
fn calculate_copy_size(&self, trade: &TradeEvent) -> f64 {
    let base_size = trade.size_usd * self.config.copy_ratio;
    
    // 限制在最小/最大范围内
    base_size.clamp(
        self.config.min_trade_size_usd,
        self.config.max_trade_size_usd,
    )
}

// 3. 风控检查
fn check_risk(&self, market: &str, side: Side, size: f64) -> bool {
    // 检查黑名单
    if self.config.blacklist.contains(&market) {
        return false;
    }
    
    // 检查持仓限制
    let current = self.get_position(market);
    if (current + size) > self.config.max_position_per_market {
        return false;
    }
    
    // 检查日亏损
    if self.daily_pnl < -self.config.max_daily_loss {
        return false;
    }
    
    true
}

// 4. 执行跟单
async fn execute_copy(&self, trade: &TradeEvent) -> Result<()> {
    let size = self.calculate_copy_size(trade);
    
    if !self.check_risk(&trade.market, trade.side, size) {
        return Ok(());
    }
    
    // 检查滑点
    let expected_price = trade.price;
    let current_price = self.get_market_price(&trade.market).await?;
    
    if (current_price - expected_price).abs() / expected_price 
        > self.config.slippage_tolerance 
    {
        warn!("Slippage too high, skipping");
        return Ok(());
    }
    
    // 执行交易
    self.place_order(&trade.market, trade.side, size).await
}
```

## ⚠️ 风险提示

1. **信息滞后** - 你看到交易时价格可能已变化
2. **被反向利用** - 聪明钱可能故意诱多/诱空
3. ** Gas 竞争** - 多人跟单时 Gas 战可能吃掉利润

## 📈 预期收益

```
跟随顶级交易员历史收益:
- 年化: 50% - 200%
- 最大回撤: 20% - 40%
- 夏普比率: 1.5 - 3.0

实际收益会因滞后和滑点降低 20%-30%
```

跟单适合新手起步，但长期建议发展自己的策略。
