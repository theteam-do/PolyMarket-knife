# PolyMarket Knife - 快速部署参考卡

## 🚀 一键部署测试

```bash
# 1. 本地编译并部署到 VPS
./scripts/deploy-to-vps.sh

# 2. 运行主网测试
./scripts/mainnet-test.sh
```

## 📋 完整流程

```bash
# 步骤 1: 本地编译
cargo build --release

# 步骤 2: 上传到 VPS
./scripts/deploy-to-vps.sh

# 步骤 3: 测试策略
./scripts/mainnet-test.sh

# 步骤 4: 查看日志
tail -f logs/*.log
```

## 🔧 常用命令

### 部署相关
```bash
# 完整部署（编译 + 上传 + 测试）
./scripts/deploy-to-vps.sh

# 只上传（不编译）
scp -i ~/works/agent-keys/agent target/release/* root@95.179.239.239:/home/de/works/PolyMarket-knife/target/release/

# 只上传配置
scp -i ~/works/agent-keys/agent config/*-mainnet-test.toml root@95.179.239.239:/home/de/works/PolyMarket-knife/config/
```

### 测试相关
```bash
# 测试所有策略
./scripts/mainnet-test.sh  # 选择 5)

# 只测试 Market Maker
./scripts/mainnet-test.sh  # 选择 1)

# 手动测试（120 秒）
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent "
    cd /home/de/works/PolyMarket-knife &&
    export POLYMARKET_PRIVATE_KEY='0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7' &&
    timeout 120 ./target/release/market-maker --config config/market-maker-mainnet-test.toml
"
```

### 日志相关
```bash
# 实时查看 VPS 日志
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent \
    "tail -f /home/de/works/PolyMarket-knife/logs/*.log"

# 下载日志
scp -i ~/works/agent-keys/agent \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/logs/*.log ./logs/

# 分析订单
grep -c "order created" logs/*.log

# 分析错误
grep -E "ERROR|panic" logs/*.log | tail -20
```

### SSH 连接
```bash
# 快速连接
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent

# 添加别名
echo "alias vps='ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent'" >> ~/.bashrc
```

## 📊 测试配置

| 策略 | 配置文件 | 订单大小 | 日亏损上限 |
|------|----------|----------|-----------|
| Market Maker | `market-maker-mainnet-test.toml` | 1 USDC | 5 USDC |
| Arbitrage | `arbitrage-mainnet-test.toml` | 1 USDC | 5 USDC |
| Follow Trade | `follow-trade-mainnet-test.toml` | 1 USDC | 5 USDC |
| Volatility Hunter | `volatility-hunter-mainnet-test.toml` | 1 USDC | 5 USDC |

## 💰 钱包信息

- **地址**: `0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6`
- **网络**: Polygon 主网
- **查看余额**: https://polygonscan.com/address/0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6

## ⚠️ 安全提醒

1. **这是真实资金测试** - 钱包只放测试所需资金
2. **测试后转移资金** - 不要留在测试钱包
3. **最大订单 1 USDC** - 已配置严格风控
4. **私钥已泄露** - 此钱包不再用于大额资金

## 🐛 修复 Bug 流程

```bash
# 1. 本地修复代码
vim src/xxx.rs

# 2. 重新编译
cargo build --release

# 3. 重新部署
./scripts/deploy-to-vps.sh

# 4. 验证修复
./scripts/mainnet-test.sh
```

## 📖 详细文档

- [部署指南](docs/DEPLOYMENT_GUIDE.md) - 完整部署流程
- [主网测试指南](docs/MAINNET_TEST_GUIDE.md) - 测试详细说明
- [VPS 调试指南](docs/VPS_DEBUG_GUIDE.md) - VPS 使用参考
- [故障排查](docs/TROUBLESHOOTING.md) - 常见问题

## 🎯 测试清单

部署前:
- [ ] 本地编译成功
- [ ] 配置文件正确
- [ ] 钱包有足够余额（MATIC ≥ 0.5, USDC ≥ 10）

部署后:
- [ ] 二进制文件上传成功
- [ ] 配置文件上传成功
- [ ] 程序可以启动

测试后:
- [ ] 订单正常创建
- [ ] 无异常错误
- [ ] 日志正常
- [ ] 钱包余额正确

---

**VPS**: 95.179.239.239  
**最后更新**: 2026-03-03
