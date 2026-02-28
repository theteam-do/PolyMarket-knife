# Order Attack - 订单攻击策略

## ⚠️⚠️⚠️ 高风险警告

**本策略利用平台机制漏洞，可能导致：**

1. **封号** - 地址被 Polymarket 永久拉黑
2. **法律风险** - 可能被起诉市场操纵
3. **社区隔离** - 被监控工具标记

**⚠️ 仅建议在测试网学习使用，严禁用于主网盈利！**

## 🎯 策略核心

利用 Polymarket"链下撮合 + 链上结算"的时间差，用注定失败的交易清空对手的订单，制造流动性真空，然后垄断价差获利。

## 📊 攻击手法

### 手法 1: 清场后垄断
1. 发起注定失败的匹配请求
2. 参与匹配的做市商订单被强制移除
3. 市场出现流动性真空
4. 攻击者挂出大幅价差订单
5. 垄断成交获利

### 手法 2: 猎杀对冲机器人
1. 用虚假成交误导机器人开仓
2. 让交易在链上失败
3. 机器人被迫平仓亏损
4. 攻击者低价接盘

## ⚡ 技术原理

```
Polymarket 撮合流程:

1. 用户 A 发起买单 (链下)
2. 用户 B 的卖单匹配 (链下)
3. 提交链上结算 (Polygon)
4. ❌ 如果链上失败 (余额不足等)
5. 所有参与匹配的订单被移除
6. 市场订单簿清空 ⚠️

攻击者利用步骤 4-5:
- 故意制造链上失败
- 清空对手订单
- 垄断价差
```

## 🔧 配置示例

```toml
# config/order-attack.toml
# ⚠️ 仅供测试网使用

[polygon]
rpc_url = "wss://mumbai-rpc.com"  # 测试网
private_key = "0x..."

[strategy]
# 攻击参数
attack_gas_limit = 50000           # 故意设置低的 Gas
attack_nonce_gap = true            # 制造 nonce 间隙
target_spread_bps = 5000           # 垄断后价差 50%

# 目标选择
min_liquidity_usd = 10000          # 最小流动性目标
exclude_addresses = []             # 排除地址

# 风险控制 (自保)
max_attacks_per_day = 10
cooldown_seconds = 300             # 攻击间隔

[warning]
testnet_only = true
acknowledged = false               # 改为 true 才能运行
```

## 🏗️ 架构设计

```
┌────────────────────────────────────────────────────────┐
│              Target Scanner                             │
│         (扫描高流动性市场)                               │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│              Attack Planner                             │
│    (选择攻击手法，计算最优参数)                          │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│              Attack Executor                            │
│    (发起注定失败的交易)                                  │
│                                                         │
│    手法:                                                │
│    - Gas 不足攻击                                       │
│    - Nonce 间隙攻击                                     │
│    - 余额不足攻击                                       │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│              Monitor                                    │
│    (监控订单簿清空状态)                                  │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│              Monopoly Trader                            │
│    (挂出垄断价差订单)                                    │
└────────────────────────────────────────────────────────┘
```

## 📝 核心逻辑

```rust
// 1. 扫描目标
fn scan_targets(&self) -> Vec<MarketInfo> {
    self.markets.iter()
        .filter(|m| m.liquidity > self.config.min_liquidity_usd)
        .filter(|m| !self.exclude_addresses.contains(&m.market_maker))
        .collect()
}

// 2. 执行攻击
async fn execute_attack(&self, target: &MarketInfo) -> Result<()> {
    // 方法 1: Gas 不足攻击
    self.send_transaction_with_low_gas(target).await?;
    
    // 方法 2: Nonce 间隙攻击
    self.send_transaction_with_nonce_gap(target).await?;
    
    // 方法 3: 余额不足攻击
    self.send_transaction_insufficient_balance(target).await?;
    
    Ok(())
}

// 3. 监控订单簿
async fn wait_for_clearing(&self, market: &str) -> bool {
    for _ in 0..10 {
        let orderbook = self.fetch_orderbook(market).await;
        if orderbook.is_empty() {
            return true;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    false
}

// 4. 垄断交易
async fn trade_monopoly(&self, market: &str) -> Result<()> {
    // 挂出大幅价差订单
    let bid = 0.10;  // 极低买价
    let ask = 0.90;  // 极高卖价
    
    self.place_order(market, Side::Buy, bid).await?;
    self.place_order(market, Side::Sell, ask).await?;
    
    // 等待被动成交
    Ok(())
}
```

## ⚠️ 防御方法

了解攻击后可以防御：

```rust
// 做市商防御策略
fn defend_against_attack(&mut self) {
    // 1. 监控异常交易模式
    if self.detect_attack_pattern() {
        // 2. 暂停做市
        self.pause_market_making();
        
        // 3. 报告平台
        self.report_to_platform();
    }
}

// 平台级防御
// - 失败订单不移除，只标记
// - 限制单地址失败次数
// - 链下预验证
```

## 📉 风险评估

| 风险类型 | 概率 | 后果 |
|----------|------|------|
| 封号 | 高 (>80%) | 永久禁止交易 |
| 法律诉讼 | 中 (30%) | 罚款/刑事责任 |
| 社区抵制 | 高 (>90%) | 被其他交易者针对 |
| 技术反制 | 中 (50%) | 策略失效 |

## 🎓 学习价值

虽然不建议使用，但理解此攻击有助于：

1. **理解撮合机制** - 深入理解 Polymarket 架构
2. **防御做市** - 保护自己免受攻击
3. **白帽研究** - 向平台报告漏洞获取奖励

**再次强调：仅供测试网学习！**
