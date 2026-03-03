# Polymarket 主网测试指南

## ⚠️ 重要安全警告

**本测试使用真实资金！**

- **钱包地址**: `0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6`
- **私钥**: `0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7`
- **网络**: Polygon 主网

**安全原则**:
1. 钱包只存放测试所需的资金
2. 测试完成后立即转移剩余资金
3. 所有策略配置为最小订单（≤1 USDC）
4. 设置严格的风控参数

## 测试前准备

### 1. 检查钱包余额

访问 [Polygonscan](https://polygonscan.com/address/0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6) 查看：
- MATIC 余额（用于 gas）
- USDC 余额（用于交易）

**最低要求**:
- MATIC ≥ 0.5（建议 1+）
- USDC ≥ 10（建议 20+）

### 2. 连接 VPS

```bash
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent
```

### 3. 设置环境变量

```bash
export POLYMARKET_PRIVATE_KEY="0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7"
export POLYMARKET_ADDRESS="0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6"
```

## 快速测试

### 方法 1: 使用自动化脚本（推荐）

在本地运行：

```bash
cd /home/de/works/PolyMarket-knife
./scripts/mainnet-test.sh
```

脚本会：
1. 检查 VPS 连接
2. 查询钱包余额
3. 让你选择要测试的策略
4. 运行测试（默认 120 秒）
5. 自动下载日志

### 方法 2: 手动测试

#### Market Maker 测试

```bash
# SSH 连接到 VPS
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent

# 进入项目目录
cd /home/de/works/PolyMarket-knife

# 设置环境变量
export POLYMARKET_PRIVATE_KEY="0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7"

# 运行测试（120 秒）
timeout 120 ./target/release/market-maker --config config/market-maker-mainnet-test.toml
```

#### Arbitrage 测试

```bash
timeout 120 ./target/release/arbitrage --config config/arbitrage-mainnet-test.toml
```

#### Follow Trade 测试

```bash
timeout 120 ./target/release/follow-trade --config config/follow-trade-mainnet-test.toml
```

#### Volatility Hunter 测试

```bash
timeout 120 ./target/release/volatility-hunter --config config/volatility-hunter-mainnet-test.toml
```

## 测试配置说明

### 所有策略的通用配置

```toml
[strategy]
order_size = 1.0           # 订单大小 1 USDC
max_position = 10          # 最大持仓 10 份

[risk]
max_order_value = 1.0      # 单笔订单最大 1 USDC
max_daily_loss = 5.0       # 日亏损上限 5 USDC
max_total_loss = 20.0      # 总亏损上限 20 USDC
```

### 配置文件位置

| 策略 | 配置文件 |
|------|----------|
| Market Maker | `config/market-maker-mainnet-test.toml` |
| Arbitrage | `config/arbitrage-mainnet-test.toml` |
| Follow Trade | `config/follow-trade-mainnet-test.toml` |
| Volatility Hunter | `config/volatility-hunter-mainnet-test.toml` |

## 测试流程

### 完整测试流程（推荐）

```bash
# 1. 运行诊断脚本
./scripts/vps-debug.sh

# 2. 检查钱包余额
# 脚本会自动显示 MATIC 和 USDC 余额

# 3. 运行主网测试
./scripts/mainnet-test.sh

# 4. 选择策略 5) 全部顺序测试
# 每个策略运行 120 秒
```

### 单个策略测试

```bash
# 只测试 Market Maker
./scripts/mainnet-test.sh
# 选择 1)

# 只测试 Arbitrage
./scripts/mainnet-test.sh
# 选择 2)
```

## 监控测试

### 实时日志

```bash
# 在 VPS 上查看实时日志
tail -f /home/de/works/PolyMarket-knife/logs/*.log
```

### 检查订单

```bash
# 搜索订单相关日志
grep -i "order.*created\|order.*filled" logs/*.log

# 搜索错误
grep -E "ERROR|panic|failed" logs/*.log
```

### 查看钱包变化

测试期间在另一个终端运行：

```bash
# 每 10 秒刷新一次余额
watch -n 10 'curl -s "https://api.polygonscan.com/api?module=account&action=balance&address=0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6&tag=latest&apikey=YourApiKey" | jq '.result''
```

## 测试后检查

### 1. 检查盈亏

```bash
# 查看日志中的 PnL 信息
grep -i "pnl\|profit\|loss" logs/*.log | tail -20
```

### 2. 检查订单状态

访问 Polymarket 查看：
- https://polymarket.com/portfolio

### 3. 平仓

如果有未平仓位，手动平仓或等待策略自动平仓。

### 4. 转移资金

测试完成后，将剩余资金转移到安全钱包：

```bash
# 使用 ethers.js 转移
node -e "
const { ethers } = require('ethers');
const provider = new ethers.providers.JsonRpcProvider('https://polygon-bor-rpc.publicnode.com');
const wallet = new ethers.Wallet('YOUR_PRIVATE_KEY', provider);
const tx = await wallet.sendTransaction({
  to: 'YOUR_SAFE_ADDRESS',
  value: await provider.getBalance(wallet.address)
});
await tx.wait();
console.log('转移完成:', tx.hash);
"
```

## 常见问题

### 1. Gas 不足

**现象**: 交易失败，报错 "out of gas"

**解决**: 充值 MATIC 到钱包

### 2. USDC 不足

**现象**: 无法下单

**解决**: 充值 USDC 到钱包（至少 10 USDC）

### 3. 连接超时

**现象**: "connection timeout"

**解决**: 
```bash
# 检查网络连接
ping clob.polymarket.com

# 检查 API 状态
curl -I https://clob.polymarket.com
```

### 4. 订单失败

**现象**: "order rejected"

**解决**:
- 检查订单大小是否超过限制
- 检查市场是否有足够流动性
- 查看日志中的详细错误信息

## 日志文件

测试日志保存在：
- 本地：`./logs/`
- VPS: `/home/de/works/PolyMarket-knife/logs/`

### 日志分析

```bash
# 统计订单数量
grep -c "order created" logs/*.log

# 统计成交数量
grep -c "order filled" logs/*.log

# 统计错误数量
grep -c "ERROR" logs/*.log

# 查看盈亏
grep "PnL" logs/*.log | tail -10
```

## 安全提醒

1. **不要修改配置文件中的风控参数**（除非你知道后果）
2. **测试时间不宜过长**（每个策略 2-5 分钟足够）
3. **测试完成后立即转移资金**
4. **不要在公共场合分享私钥**
5. **定期更换测试钱包**

## 支持

遇到问题时：
1. 查看日志文件
2. 运行诊断脚本 `./scripts/vps-debug.sh`
3. 检查钱包余额和交易历史

---

**最后更新**: 2026-03-03
**测试钱包**: `0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6`
