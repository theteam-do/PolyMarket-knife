# 测试网验证指南

## 1. 获取测试 USDC

### Polygon Mumbai 测试网

1. **获取测试 MATIC**
   - https://faucet.polygon.technology/
   - 每次可领取 0.5 MATIC
   - 用于支付 Gas 费用

2. **获取测试 USDC**
   - 地址：0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 (主网)
   - 测试网：需要桥接或从测试网水龙头获取
   - 或使用测试网 DEX 兑换

## 2. 配置测试网

### 复制配置文件

```bash
cd /home/de/works/PolyMarket-knife

# Market Maker
cp config/market-maker-testnet.toml config/market-maker.toml

# Arbitrage
cp config/arbitrage-testnet.toml config/arbitrage.toml
```

### 设置私钥

```bash
# 方法 1: 编辑配置文件
vim config/market-maker.toml
# 修改 private_key = "YOUR_TESTNET_PRIVATE_KEY"

# 方法 2: 使用环境变量 (推荐)
export POLYMARKET_PRIVATE_KEY="your_testnet_private_key"
```

### 测试网参数

| 网络 | RPC URL | CLOB Host |
|------|---------|-----------|
| Mumbai | https://rpc-mumbai.maticvigil.com | https://testnet-clob.polymarket.com |
| Polygon 主网 | https://polygon-bor-rpc.publicnode.com | https://clob.polymarket.com |

## 3. 运行测试

### Market Maker 测试

```bash
# 设置环境变量
export POLYMARKET_PRIVATE_KEY="your_key"

# 运行测试
./target/release/market-maker --config config/market-maker-testnet.toml

# 观察日志
# INFO Market Maker starting...
# INFO Monitoring X markets
# INFO Orders placed for market_id: buy=xxx sell=xxx
```

### Arbitrage 测试

```bash
./target/release/arbitrage --config config/arbitrage-testnet.toml

# 观察日志
# INFO Arbitrageur starting...
# INFO Arbitrage opportunity detected: BuyAndMint...
# INFO Arbitrage executed, profit: $X
```

### Follow Trade 测试

```bash
./target/release/follow-trade --config config/follow-trade.toml

# 需要配置聪明钱地址
# smart_addresses = ["0x..."]
```

### Volatility Hunter 测试

```bash
./target/release/volatility-hunter --config config/volatility-hunter.toml

# 需要配置币安 API
# api_key = "your_binance_key"
```

## 4. 验证检查清单

### 基础功能

- [ ] 程序启动成功
- [ ] 连接到测试网 CLOB
- [ ] 认证成功
- [ ] 获取订单簿成功
- [ ] 下单成功
- [ ] 撤单成功

### 监控指标

- [ ] PnL 指标更新
- [ ] 订单指标更新
- [ ] 告警系统工作
- [ ] 仪表板显示正常

### 风控检查

- [ ] 日亏损限制生效
- [ ] 持仓限制生效
- [ ] 连续亏损停止交易

## 5. 常见问题

### Q: 如何获取测试网私钥？

A: 使用 MetaMask 创建新账户，切换到 Mumbai 测试网，导出私钥。

### Q: 订单一直不成交？

A: 测试网流动性较低，调整价格或等待。

### Q: Gas 费用过高？

A: 调整 gas_price_gwei 参数，测试网通常 50 Gwei 足够。

### Q: 认证失败？

A: 检查私钥格式，确保有 0x 前缀。

## 6. 测试报告模板

```markdown
## 测试报告

**日期**: 2026-03-01
**策略**: Market Maker
**网络**: Mumbai Testnet

### 测试结果

- [ ] 启动成功
- [ ] 认证成功
- [ ] 下单成功
- [ ] 撤单成功
- [ ] 监控正常

### 性能数据

- 平均延迟：XX ms
- 下单成功率：XX%
- 日盈亏：$XX

### 问题记录

1. 问题描述
2. 解决方案

### 改进建议

1. 建议内容
```

## 7. 下一步

测试通过后：
1. 调整参数优化性能
2. 增加资金进行实盘测试
3. 监控实际收益

