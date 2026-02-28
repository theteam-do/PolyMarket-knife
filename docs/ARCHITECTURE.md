# PolyMarket Knife 架构文档

## 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        PolyMarket Knife                          │
│                     6 个独立 Rust 程序                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ market-      │  │ arbitrage    │  │ follow-      │          │
│  │ maker        │  │              │  │ trade        │          │
│  │ 返佣做市      │  │ 套利         │  │ 跟单         │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ volatility-  │  │ info-        │  │ order-       │          │
│  │ hunter       │  │ edge         │  │ attack       │          │
│  │ 波动狩猎      │  │ 信息差        │  │ 订单攻击      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 共享设计原则

### 1. 极致性能

所有程序遵循以下优化原则：

- **单线程事件循环** - 避免锁竞争和上下文切换
- **零拷贝数据处理** - 使用 `bytes` crate
- **预分配内存** - 对象池复用
- **LTO 编译** - 链接时优化
- **panic = abort** - 减小二进制大小

### 2. 低复杂度

- **模块化设计** - 每个模块职责单一
- **配置驱动** - 所有参数可配置
- **无状态核心** - 便于测试和扩展
- **错误处理** - 使用 `anyhow`/`thiserror`

### 3. 可观测性

- **结构化日志** - `tracing` + JSON 格式
- **Prometheus 指标** - 所有关键指标暴露
- **健康检查** - HTTP 端点监控

## 程序间对比

| 程序 | 延迟要求 | 线程模型 | 数据源 | 复杂度 |
|------|----------|----------|--------|--------|
| market-maker | <100ms | 单线程 | Poly WS | 中 |
| arbitrage | <50ms | 单线程 | Poly API | 低 |
| follow-trade | <500ms | 单线程 | Chain Events | 低 |
| volatility-hunter | <20ms | 多线程 | Binance WS + Poly WS | 高 |
| info-edge | <1s | 多线程 | News APIs | 中 |
| order-attack | <100ms | 单线程 | Poly API | 高 |

## 依赖关系

```
所有程序共享以下依赖:
├── tokio          # 异步运行时
├── serde          # 序列化
├── reqwest        # HTTP 客户端
├── tracing        # 日志
└── anyhow         # 错误处理

特定程序额外依赖:
├── market-maker   ──┬── tungstenite (WS)
│                    └── ethers (EVM)
├── volatility-hunter ──┬── crossbeam (并发)
│                       └── rust_decimal (精度)
└── info-edge      ──── regex (NLP)
```

## 部署架构

```
AWS us-east-1 (推荐)
├── EC2 c5.large (低延迟优化)
│   ├── CPU 隔离: isolated=2,3
│   ├── CPU 绑定: taskset -c 2,3
│   └── 网络优化: ethtool -K eth0 gro off
│
├── 运行策略
│   ├── market-maker (CPU 2)
│   ├── volatility-hunter (CPU 3)
│   └── 其他策略 (CPU 0,1)
│
└── 监控
    ├── Prometheus
    ├── Grafana
    └── Alertmanager
```

## 安全注意事项

1. **私钥管理** - 使用环境变量或密钥管理服务
2. **配置文件** - 不要提交到版本控制
3. **网络隔离** - 使用 VPC 和安全组
4. **监控告警** - 异常交易立即通知

## 扩展新策略

添加新策略的步骤：

1. 复制模板目录
2. 修改 `Cargo.toml` 名称
3. 实现核心逻辑
4. 添加配置文件
5. 更新主 `README.md`

```bash
# 模板
cp -r market-maker new-strategy
cd new-strategy
# 修改代码和配置
```
