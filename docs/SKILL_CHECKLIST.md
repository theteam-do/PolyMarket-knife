# Skill 配置完整性检查清单

## ✅ 已记录的关键信息

### 1. VPS 服务器配置 ✅
- [x] 主机 IP: `95.179.239.239`
- [x] 用户：`root`
- [x] SSH 密钥：`~/works/agent-keys/agent`
- [x] 连接命令：`ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent`
- [x] 项目路径：`/home/de/works/PolyMarket-knife`

### 2. Polymarket 钱包配置 ✅
- [x] 钱包地址：`0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6`
- [x] 私钥：`0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7`
- [x] 网络：Polygon 主网
- [x] 安全警告：私钥已明文，仅用于测试

### 3. 测试策略配置 ✅
- [x] 最大订单：1 USDC
- [x] 买卖数量：1 份
- [x] 日亏损上限：5 USDC
- [x] 总亏损上限：20 USDC
- [x] 测试原则：最小金额、快速验证、及时转移

### 4. 工作流程 ✅
- [x] 本地编译（不推荐 VPS 编译）
- [x] 上传二进制到 VPS
- [x] VPS 运行测试
- [x] 下载日志分析
- [x] 本地修复 bug
- [x] 重新编译部署

### 5. 脚本配置 ✅
- [x] `deploy-to-vps.sh` - 本地编译并部署
- [x] `mainnet-test.sh` - 运行主网测试
- [x] `vps-debug.sh` - 完整诊断
- [x] `vps-quick-connect.sh` - 快速连接

### 6. 配置文件 ✅
- [x] `config/market-maker-mainnet-test.toml`
- [x] `config/arbitrage-mainnet-test.toml`
- [x] `config/follow-trade-mainnet-test.toml`
- [x] `config/volatility-hunter-mainnet-test.toml`

### 7. 文档 ✅
- [x] `docs/DEPLOYMENT_GUIDE.md` - 部署指南
- [x] `docs/MAINNET_TEST_GUIDE.md` - 主网测试指南
- [x] `docs/VPS_DEBUG_GUIDE.md` - VPS 调试指南
- [x] `QUICK_REFERENCE.md` - 快速参考

### 8. 安全提醒 ✅
- [x] 私钥已泄露警告
- [x] 测试后转移资金
- [x] 不要存放超过测试所需资金
- [x] 使用最小订单测试

## 📋 使用场景

### 场景 1: 快速部署测试
```bash
./scripts/deploy-to-vps.sh
./scripts/mainnet-test.sh
```

### 场景 2: 修复 Bug 后重新部署
```bash
# 1. 本地修复代码
# 2. 重新编译
cargo build --release
# 3. 重新部署
./scripts/deploy-to-vps.sh
# 4. 验证修复
./scripts/mainnet-test.sh
```

### 场景 3: AI 辅助调试
告诉 AI："帮我调试 VPS 上的程序"
AI 会加载 `vps-debugger` skill，使用你的配置自动诊断

## 🎯 测试目的

1. ✅ 验证生产级真实流程
2. ✅ 发现并修复 bug
3. ✅ 准备生产部署
4. ✅ 最小成本测试（≤1 USDC 订单）

## ⚠️ 重要提醒

1. **不要**在 VPS 上编译（配置低、慢、可能 OOM）
2. **必须**本地编译后上传
3. **测试后**立即转移资金
4. **不要**在测试钱包存放大资金

---

**检查时间**: 2026-03-03  
**检查者**: AI Assistant  
**状态**: ✅ 所有关键信息已记录到 skill
