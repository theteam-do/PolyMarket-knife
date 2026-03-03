# 主网部署验证清单

**重要**: 主网操作涉及真实资金，请逐项验证 ✅

---

## ⚠️ 风险警告

在继续之前，请确认你理解以下风险：

- [ ] **资金损失风险**: 程序可能存在 Bug，导致资金损失
- [ ] **智能合约风险**: Polymarket 合约可能存在漏洞
- [ ] **技术风险**: 网络延迟、API 故障可能导致交易失败
- [ ] **市场风险**: 价格波动可能导致亏损
- [ ] **安全风险**: 私钥泄露可能导致资金被盗

**建议**: 先用小额资金测试，确认稳定后再增加投入

---

## 📋 验证清单

### 1. 环境准备

- [ ] **Rust 环境**: `rustc --version` >= 1.75
- [ ] **系统依赖**: `build-essential`, `pkg-config`, `libssl-dev` 已安装
- [ ] **编译通过**: `cargo build --release` 成功
- [ ] **测试通过**: `cargo test --release` 全部通过

### 2. 钱包准备

- [ ] **Polygonscan 验证**: 钱包地址可在 https://polygonscan.com 查询
- [ ] **USDC 余额**: 账户有足够 USDC (建议 >= $1000 起步)
- [ ] **POL 余额**: 账户有足够 POL 支付 Gas (建议 >= 10 POL)
- [ ] **私钥安全**: 私钥仅存储在本地，未上传到任何地方

### 3. 配置验证

#### 3.1 执行模式配置

**必须设置**:
```toml
[execution]
mode = "live"              # ⚠️ 从 paper 改为 live
environment = "mainnet"    # ⚠️ 从 testnet 改为 mainnet
live_acknowledged = true   # ⚠️ 必须显式确认
live_failure_fallback_to_paper = true
```

#### 3.2 RPC 配置

**主网 RPC**:
```toml
[polygon]
rpc_url = "https://polygon-bor-rpc.publicnode.com"
# 或使用付费 RPC 提高稳定性
# rpc_url = "https://polygon-mainnet.g.alchemy.com/v2/YOUR_KEY"
```

#### 3.3 CLOB 配置

**主网 CLOB**:
```toml
[clob]
host = "https://clob.polymarket.com"  # ⚠️ 主网地址
```

### 4. 策略特定验证

#### 4.1 Arbitrage (套利)

- [ ] **最小利润**: `min_profit_usd >= 0.02` (覆盖 Gas 成本)
- [ ] **单笔最大**: `max_position_per_trade` 设置合理
- [ ] **Gas 价格**: `gas_price_gwei` 设置合理 (当前网络水平)
- [ ] **排除市场**: 设置 `exclude_market_ids` 排除高风险市场

**验证命令**:
```bash
./target/release/arbitrage config/arbitrage-mainnet.toml
```

**预期日志**:
```
INFO Config loaded: mode=Live environment=Mainnet live_ack=true
INFO [LIVE] Arbitrage execution: opportunity=...
```

#### 4.2 Follow Trade (跟单)

- [ ] **聪明钱地址**: 设置真实有效的聪明钱地址
- [ ] **跟单比例**: `copy_ratio` 设置合理 (建议 0.05-0.2)
- [ ] **滑点容忍**: `slippage_tolerance` 设置合理 (建议 0.02-0.05)
- [ ] **风控限制**: `max_position_per_market` 和 `max_daily_loss` 设置

**验证命令**:
```bash
./target/release/follow-trade config/follow-trade-mainnet.toml
```

**预期日志**:
```
INFO Config loaded: mode=Live environment=Mainnet live_ack=true
INFO Monitoring X smart addresses
INFO [LIVE] Copy trade: market=...
```

#### 4.3 Market Maker (做市)

- [ ] **市场 ID**: 设置真实存在的市场 token ID
- [ ] **订单大小**: `order_size_usd` 设置合理
- [ ] **价差设置**: `spread_bps` 设置合理 (建议 100-200)
- [ ] **风控限制**: `max_position_usd` 和 `max_loss_per_day` 设置

**验证命令**:
```bash
./target/release/market-maker config/market-maker-mainnet.toml
```

**预期日志**:
```
INFO Market Maker starting...
INFO Monitoring X markets
INFO Orders placed for 0x...: buy=..., sell=...
```

### 5. 安全验证

#### 5.1 私钥安全

- [ ] **环境变量**: 私钥通过 `POLYMARKET_PRIVATE_KEY` 环境变量设置
- [ ] **文件权限**: 配置文件权限设置为 `600`
- [ ] **版本控制**: 配置文件已添加到 `.gitignore`

**设置方法**:
```bash
# 设置私钥环境变量
export POLYMARKET_PRIVATE_KEY="0x你的私钥"

# 设置配置文件权限
chmod 600 config/*.toml
chmod 600 .env*
```

#### 5.2 执行确认

**启动时必须看到**:
```
INFO Config loaded: mode=Live environment=Mainnet live_ack=true
```

**如果看到 mode=Paper，说明配置有误！**

### 6. 监控验证

#### 6.1 日志监控

- [ ] **日志级别**: 设置合理的 `RUST_LOG` (建议 `info`)
- [ ] **日志文件**: 配置日志输出到文件
- [ ] **告警设置**: 设置异常告警

**启动命令**:
```bash
RUST_LOG=info ./target/release/arbitrage config/arbitrage-mainnet.toml 2>&1 | tee arbitrage.log
```

#### 6.2 指标监控

- [ ] **Prometheus**: 访问 `http://localhost:9090/metrics`
- [ ] **关键指标**: PnL、订单数、失败率

**验证命令**:
```bash
curl http://localhost:9090/metrics
```

### 7. 小额测试

**强烈建议先用小额测试 1-2 周**

#### 7.1 测试步骤

1. **第 1-3 天**: 极小额测试 ($10-50/笔)
   - [ ] 验证下单成功
   - [ ] 验证成交正常
   - [ ] 验证 PnL 计算准确

2. **第 4-7 天**: 小额测试 ($50-200/笔)
   - [ ] 验证风控生效
   - [ ] 验证日亏损限制
   - [ ] 验证取消订单正常

3. **第 2 周**: 中额测试 ($200-500/笔)
   - [ ] 验证策略稳定性
   - [ ] 验证网络延迟影响
   - [ ] 验证异常处理

4. **第 3 周起**: 根据测试结果调整

### 8. 应急响应

#### 8.1 紧急停止

**所有策略都支持**:
```bash
# Ctrl+C 停止
# 会自动取消所有挂单
```

#### 8.2 手动取消

**Polymarket CLOB**:
1. 访问 https://polymarket.com
2. 进入 Portfolio -> Orders
3. 手动取消所有订单

#### 8.3 联系人

- [ ] 技术支持联系方式
- [ ] 紧急联系人

---

## 📊 主网配置示例

### Arbitrage 主网配置

```toml
# config/arbitrage-mainnet.toml

[polygon]
rpc_url = "https://polygon-bor-rpc.publicnode.com"
# 私钥通过环境变量 POLYMARKET_PRIVATE_KEY 设置

[clob]
host = "https://clob.polymarket.com"

[strategy]
min_profit_usd = 0.02
max_position_per_trade = 500    # 小额测试
scan_interval_ms = 50
gas_price_gwei = 100            # 主网适当提高
include_all = true
exclude_market_ids = []

[execution]
mode = "live"
environment = "mainnet"
live_acknowledged = true
live_failure_fallback_to_paper = true
```

### Follow Trade 主网配置

```toml
# config/follow-trade-mainnet.toml

[polygon]
rpc_url = "https://polygon-bor-rpc.publicnode.com"

[clob]
host = "https://clob.polymarket.com"

[strategy]
smart_addresses = [
    "0x你的聪明钱地址 1",
    "0x你的聪明钱地址 2",
]
min_trade_size_usd = 50
max_trade_size_usd = 500
copy_ratio = 0.1              # 10% 跟单
slippage_tolerance = 0.03     # 3% 滑点
max_position_per_market = 2000
max_daily_loss = 500
blacklist = []

[execution]
mode = "live"
environment = "mainnet"
live_acknowledged = true
live_failure_fallback_to_paper = true
```

### Market Maker 主网配置

```toml
# config/market-maker-mainnet.toml

[polygon]
rpc_url = "https://polygon-bor-rpc.publicnode.com"

[clob]
host = "https://clob.polymarket.com"

[strategy]
market_ids = [
    "市场 token ID 1",    # 替换为真实市场
    "市场 token ID 2",
]
spread_bps = 150          # 1.5% 价差
min_spread_bps = 100
max_spread_bps = 300
order_size_usd = 200      # 小额测试
refresh_interval_ms = 100
skew_inventory = true

[risk]
max_position_usd = 5000
max_loss_per_day = 300
stop_loss_pct = 5.0
max_orders = 10
max_order_size_usd = 500

[execution]
mode = "live"
environment = "mainnet"
live_acknowledged = true
live_failure_fallback_to_paper = true
```

---

## ✅ 启动前最后检查

在首次启动主网前，请再次确认：

- [ ] **配置正确**: `mode=live`, `environment=mainnet`, `live_acknowledged=true`
- [ ] **私钥安全**: 通过环境变量设置，文件权限 600
- [ ] **余额充足**: USDC >= $1000, POL >= 10
- [ ] **小额测试**: 首次使用建议 <= $100/笔
- [ ] **监控就绪**: 日志输出正常，可实时查看
- [ ] **应急准备**: 知道如何紧急停止

---

## 📞 获取帮助

遇到问题时：

1. **检查日志**: 查看错误信息
2. **检查配置**: 确认所有参数正确
3. **检查网络**: 确认 RPC 和 CLOB 连接正常
4. **降低金额**: 先用更小金额测试

---

**最后更新**: 2026-03-01  
**版本**: 1.0

**免责声明**: 本清单仅供参考，不构成投资建议。主网交易风险自负。
