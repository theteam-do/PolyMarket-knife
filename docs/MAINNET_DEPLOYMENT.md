# 主网部署快速指南

**版本**: 1.0  
**更新日期**: 2026-03-01

---

## 🚀 快速开始 (5 分钟)

### 步骤 1: 设置私钥

```bash
# 设置环境变量 (当前会话有效)
export POLYMARKET_PRIVATE_KEY="0x你的私钥"

# 或添加到 ~/.bashrc (永久有效)
echo 'export POLYMARKET_PRIVATE_KEY="0x你的私钥"' >> ~/.bashrc
source ~/.bashrc
```

### 步骤 2: 运行验证脚本

```bash
cd /home/de/works/PolyMarket-knife
./scripts/verify-mainnet.sh
```

**预期输出**:
```
✓ Rust 版本：1.93.0
✓ 编译成功
✓ 私钥已设置
✓ Arbitrage: mode=live
✓ Arbitrage: environment=mainnet
✓ 验证通过 - 可以部署主网
```

### 步骤 3: 启动策略

```bash
# Arbitrage (套利)
./target/release/arbitrage config/arbitrage-mainnet.toml

# Follow Trade (跟单)
./target/release/follow-trade config/follow-trade-mainnet.toml

# Market Maker (做市)
./target/release/market-maker config/market-maker-mainnet.toml
```

---

## 📋 部署前检查清单

### ✅ 必须完成

- [ ] **私钥设置**: `echo $POLYMARKET_PRIVATE_KEY` 显示非空
- [ ] **钱包余额**: USDC >= $1000, POL >= 10
- [ ] **配置验证**: `./scripts/verify-mainnet.sh` 通过
- [ ] **小额测试**: 首次运行建议 <= $100/笔

### ⚠️ 强烈建议

- [ ] **付费 RPC**: 使用 Alchemy/Infura 提高稳定性
- [ ] **监控设置**: 日志输出到文件，便于排查问题
- [ ] **告警配置**: 设置异常通知

---

## 🔧 配置说明

### Arbitrage (套利)

**适合**: 保守型，追求稳定收益

**配置要点**:
```toml
[strategy]
min_profit_usd = 0.05      # 最小利润 (覆盖 Gas)
max_position_per_trade = 500  # 单笔最大 (建议小额)
gas_price_gwei = 100       # Gas 价格 (主网适当提高)
```

**预期收益**: 20%-50%/年  
**风险等级**: ⭐ (低)

---

### Follow Trade (跟单)

**适合**: 新手友好，跟随聪明钱

**配置要点**:
```toml
[strategy]
smart_addresses = [        # 聪明钱地址 (必须替换)
    "0x 真实有效的地址 1",
    "0x 真实有效的地址 2",
]
copy_ratio = 0.1           # 跟单比例 (10%)
slippage_tolerance = 0.03  # 滑点容忍 (3%)
```

**如何找聪明钱**:
1. 访问 https://polymarket.com
2. 查看高盈利用户的交易记录
3. 复制其钱包地址

**预期收益**: 50%-150%/年  
**风险等级**: ⭐⭐ (中)

---

### Market Maker (做市)

**适合**: 有经验交易者，追求稳定现金流

**配置要点**:
```toml
[strategy]
market_ids = [             # 市场 ID (必须替换)
    "0x 真实市场 token ID",
]
spread_bps = 150           # 价差 (1.5%)
order_size_usd = 200       # 订单大小 (建议小额)
```

**如何找市场 ID**:
1. 访问 https://polymarket.com
2. 选择活跃市场
3. 从 URL 或 API 获取 token ID

**预期收益**: 30%-80%/年  
**风险等级**: ⭐⭐ (中)

---

## 📊 监控和日志

### 查看实时日志

```bash
# 启动时输出到文件
./target/release/arbitrage config/arbitrage-mainnet.toml 2>&1 | tee arbitrage.log

# 实时查看日志
tail -f arbitrage.log
```

### 关键日志信息

**正常启动**:
```
INFO Config loaded: mode=Live environment=Mainnet live_ack=true
INFO Arbitrage starting...
INFO Scanning markets from: ...
```

**发现机会**:
```
INFO [LIVE] Arbitrage execution: opportunity=BuyAndMint profit=$0.05
INFO BuyAndMint executed: shares=100 profit=$5.00
```

**异常情况**:
```
WARN Live execution failed: ... Falling back to paper mode
ERROR Risk check failed: daily loss limit reached
```

### 监控指标

```bash
# 查看 Prometheus 指标
curl http://localhost:9090/metrics

# 关键指标
# - daily_pnl_usd: 日 PnL
# - orders_placed_total: 下单数
# - orders_filled_total: 成交数
```

---

## 🛑 紧急停止

### 方法 1: Ctrl+C (推荐)

```bash
# 在运行策略的终端按 Ctrl+C
# 会自动取消所有挂单
```

### 方法 2: 杀死进程

```bash
# 查找进程
ps aux | grep arbitrage

# 杀死进程
kill <PID>
```

### 方法 3: 手动取消订单

1. 访问 https://polymarket.com
2. 登录钱包
3. 进入 Portfolio -> Orders
4. 点击 Cancel All

---

## 💡 最佳实践

### 1. 小额测试 (第 1-2 周)

```toml
# 建议初始配置
max_position_per_trade = 100  # $100/笔
max_daily_loss = 200          # $200/天
```

### 2. 逐步增加 (第 3-4 周)

根据测试结果调整：
- 盈利稳定 → 增加仓位
- 频繁亏损 → 降低仓位或调整策略

### 3. 定期复盘

每周检查：
- 总 PnL
- 胜率
- 最大回撤
- 失败原因

### 4. 风险控制

**永远不要**:
- ❌ 投入超过承受能力的资金
- ❌ 设置过高的仓位
- ❌ 忽略日亏损限制
- ❌ 在不稳定网络环境运行

**一定要**:
- ✅ 设置严格的止损
- ✅ 定期提取利润
- ✅ 保持监控
- ✅ 及时更新配置

---

## 🔒 安全提醒

### 私钥安全

- ✅ 使用环境变量存储私钥
- ✅ 配置文件权限设置为 600
- ✅ 不要将私钥提交到 Git
- ❌ 不要在配置文件明文存储私钥
- ❌ 不要将私钥发送给任何人

### 网络安全

- ✅ 使用 VPS 部署 (推荐 AWS us-east-1)
- ✅ 配置防火墙规则
- ✅ 使用私有网络
- ❌ 不要在公共 WiFi 运行

### 资金安全

- ✅ 使用专用钱包 (不要与主钱包混用)
- ✅ 只存入必要资金
- ✅ 定期提取利润
- ❌ 不要投入全部资金

---

## 📞 故障排查

### 问题 1: "live mode requires explicit acknowledgement"

**原因**: 配置中 `live_acknowledged` 未设置为 true

**解决**:
```toml
[execution]
live_acknowledged = true
```

### 问题 2: "Failed to connect to RPC"

**原因**: RPC 节点不可达

**解决**:
```toml
[polygon]
# 更换 RPC 节点
rpc_url = "https://polygon-mainnet.g.alchemy.com/v2/YOUR_KEY"
```

### 问题 3: "Insufficient balance"

**原因**: USDC 或 POL 余额不足

**解决**:
1. 充值 USDC 到钱包
2. 充值 POL 支付 Gas
3. 降低 `max_position_per_trade`

### 问题 4: "Order rejected"

**原因**: 订单参数不符合要求

**解决**:
1. 检查 `spread_bps` 是否合理
2. 检查 `order_size_usd` 是否超过限制
3. 查看 Polymarket 订单要求

---

## 📚 相关文档

- [MAINNET_CHECKLIST.md](MAINNET_CHECKLIST.md) - 完整验证清单
- [CONFIGURATION.md](CONFIGURATION.md) - 配置详解
- [FAQ.md](FAQ.md) - 常见问题

---

## ⚠️ 免责声明

**重要提醒**:

1. 本软件仅供学习研究使用
2. 不构成任何投资建议
3. 使用本软件进行交易的风险由用户自行承担
4. 过往收益不代表未来表现
5. 请遵守当地法律法规

**交易有风险，投资需谨慎**

---

**最后更新**: 2026-03-01  
**支持**: 遇到问题请查看文档或联系技术支持
