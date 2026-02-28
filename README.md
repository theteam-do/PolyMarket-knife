# PolyMarket Knife 🔪

Polymarket 极致性能交易工具集 - 6 个独立 Rust 程序，每个针对特定策略优化

## 📊 策略全景

| 程序 | 策略类型 | 风险 | 预期收益 | 延迟要求 |
|------|----------|------|----------|----------|
| `market-maker` | 返佣做市 | ⭐⭐ | 5%~20%/年 | <100ms |
| `arbitrage` | 套利 | ⭐ | 稳定薄利 | <50ms |
| `follow-trade` | 跟单 | ⭐⭐ | 跟随高手 | <500ms |
| `volatility-hunter` | 波动狩猎 | ⭐⭐⭐⭐ | 单日 8 万刀 | <10ms |
| `info-edge` | 信息差 | ⭐⭐⭐ | 1200%+ | <1s |
| `order-attack` | 订单攻击 | ⭐⭐⭐⭐⭐ | 单日 1.6 万刀 | <20ms |

## 🚀 快速开始

```bash
# 编译所有程序
cargo build --release

# 运行特定策略
./target/release/market-maker --config config/market-maker.toml
./target/release/arbitrage --config config/arbitrage.toml
./target/release/follow-trade --config config/follow-trade.toml
./target/release/volatility-hunter --config config/volatility-hunter.toml
./target/release/info-edge --config config/info-edge.toml
./target/release/order-attack --config config/order-attack.toml
```

## 📁 目录结构

```
PolyMarket-knife/
├── market-maker/          # 返佣做市策略
├── arbitrage/             # 套利策略
├── follow-trade/          # 跟单策略
├── volatility-hunter/     # 波动狩猎策略
├── info-edge/             # 信息差交易策略
├── order-attack/          # 订单攻击策略
├── docs/                  # 详细文档
├── scripts/               # 部署/监控脚本
└── config/                # 配置文件模板
```

## ⚡ 性能优化要点

所有程序共享以下优化原则：

1. **零拷贝数据处理** - 使用 `bytes` crate 避免内存复制
2. **无锁架构** - 单线程事件循环，避免锁竞争
3. **预分配内存** - 对象池复用，避免运行时分配
4. **内核旁路** - 支持 DPDK/XDP（可选）
5. **CPU 亲和性** - 绑定核心，避免上下文切换

## ⚠️ 风险警告

- **订单攻击** 可能导致封号，仅建议在测试网使用
- **信息差交易** 可能涉及法律风险，请自行评估
- 所有策略均有亏损可能，请做好风控

## 📄 各策略详细文档

- [返佣做市](market-maker/README.md)
- [套利](arbitrage/README.md)
- [跟单](follow-trade/README.md)
- [波动狩猎](volatility-hunter/README.md)
- [信息差](info-edge/README.md)
- [订单攻击](order-attack/README.md)

## 🔧 依赖

- Rust 1.75+
- OpenSSL
- pkg-config

## 📝 License

MIT
