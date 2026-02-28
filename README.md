# PolyMarket Knife 🔪

[![Build Status](https://github.com/theteam-do/PolyMarket-knife/actions/workflows/ci.yml/badge.svg)](https://github.com/theteam-do/PolyMarket-knife/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/Rust-1.75+-blue.svg)](https://www.rust-lang.org)

**Polymarket 交易工具集** - 6 个完整的自动化交易策略，基于官方 SDK 构建

## 🎯 策略总览

| 策略 | 风险 | 预期收益 | 适合人群 | 状态 |
|------|------|----------|----------|------|
| **Market Maker** | ⭐⭐ | 30%-80%/年 | 有经验交易者 | ✅ 完成 |
| **Arbitrage** | ⭐ | 20%-50%/年 | 保守型 | ✅ 完成 |
| **Follow Trade** | ⭐⭐ | 50%-150%/年 | 新手友好 | ✅ 完成 |
| **Volatility Hunter** | ⭐⭐⭐⭐ | 100%-500%/年 | 专业交易者 | ✅ 完成 |
| **Info Edge** | ⭐⭐⭐ | 200%+/年 | 有信息优势 | ✅ 完成 |
| **Order Attack** | ⭐⭐⭐⭐⭐ | 测试网学习 | 技术探索者 | ✅ 完成 |

## 🚀 快速开始

### 1. 安装依赖

```bash
# Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 系统依赖
sudo apt install build-essential pkg-config libssl-dev
```

### 2. 克隆项目

```bash
git clone https://github.com/theteam-do/PolyMarket-knife.git
cd PolyMarket-knife
```

### 3. 编译

```bash
# 编译所有策略
cargo build --release

# 二进制文件位于
./target/release/market-maker
./target/release/arbitrage
./target/release/follow-trade
./target/release/volatility-hunter
./target/release/info-edge
./target/release/order-attack
```

### 4. 配置

```bash
# 复制配置模板
cp config/market-maker.toml.example config/market-maker.toml

# 编辑配置
vim config/market-maker.toml

# 设置私钥 (推荐环境变量)
export POLYMARKET_PRIVATE_KEY="your_private_key"
```

### 5. 运行

```bash
# 运行做市商
./target/release/market-maker --config config/market-maker.toml

# 运行套利
./target/release/arbitrage --config config/arbitrage.toml

# 运行跟单
./target/release/follow-trade --config config/follow-trade.toml
```

## 📚 文档

### 入门指南

- [快速开始](QUICKSTART.md) - 5 分钟上手
- [策略选择](docs/STRATEGY_GUIDE.md) - 选择适合你的策略
- [测试网验证](docs/TESTNET_GUIDE.md) - 测试网使用指南

### 技术文档

- [架构设计](docs/ARCHITECTURE.md) - 系统架构
- [API 集成](docs/API_INTEGRATION.md) - 官方 SDK 使用
- [监控告警](docs/MONITORING_GUIDE.md) - 监控指标和告警
- [性能优化](docs/PERFORMANCE_OPTIMIZATION.md) - 性能调优

### 参考文档

- [部署指南](docs/DEPLOYMENT.md) - 生产环境部署
- [常见问题](docs/FAQ.md) - FAQ
- [贡献指南](CONTRIBUTING.md) - 如何贡献

## 📊 监控指标

系统提供完整的监控指标：

- **PnL**: 日盈亏、总盈亏、最大回撤
- **订单**: 下单数、成交数、取消数、失败数
- **性能**: API 延迟、下单延迟
- **风控**: 持仓、风险敞口、连续亏损

### Prometheus 集成

```yaml
scrape_configs:
  - job_name: 'polymarket'
    static_configs:
      - targets: ['localhost:9090']
```

### Grafana 仪表板

访问 `http://localhost:3000` 查看实时监控。

## ⚠️ 风险提示

1. **交易风险**: 所有策略都有亏损可能
2. **技术风险**: 程序可能存在 Bug
3. **法律风险**: 某些策略可能受当地法律限制
4. **安全风险**: 私钥泄露可能导致资金损失

**建议**:
- 先用测试网验证
- 从小资金开始
- 设置合理的风控参数
- 定期监控系统状态

## 🛠️ 技术栈

- **语言**: Rust 2021 Edition
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **WebSocket**: Tokio-Tungstenite
- **序列化**: Serde
- **区块链**: Alloy (官方 SDK)
- **监控**: Prometheus

## 🤝 贡献

欢迎贡献！请查看 [贡献指南](CONTRIBUTING.md)。

### 贡献者

- [@theteam-do](https://github.com/theteam-do) - 初始实现

## 📄 License

MIT License - 查看 [LICENSE](LICENSE) 文件

## 🔗 链接

- **GitHub**: https://github.com/theteam-do/PolyMarket-knife
- **Polymarket**: https://polymarket.com
- **官方文档**: https://docs.polymarket.com
- **Discord**: [加入社区](TODO)

---

**免责声明**: 本项目仅供学习研究使用，不构成投资建议。使用本软件进行交易的风险由用户自行承担。
