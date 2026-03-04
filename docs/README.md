# PolyMarket Knife 文档

本项目文档包含使用指南、API 参考和最佳实践。

## 📚 文档目录

### 核心文档

- [错误处理指南](ERROR_HANDLING.md) - 错误处理最佳实践
- [安全指南](../SECURITY.md) - 安全和密钥管理
- [依赖升级计划](../DEPENDENCY_UPGRADE_PLAN.md) - 依赖维护和升级

### 快速开始

- [README](../README.md) - 项目介绍
- [QUICKSTART](../QUICKSTART.md) - 快速开始指南
- [CONTRIBUTING](../CONTRIBUTING.md) - 贡献指南

### 策略文档

- [做市策略](../market-maker/) - 做市商实现
- [套利策略](../arbitrage/) - 套利检测和执行
- [跟单策略](../follow-trade/) - 智能钱跟单
- [波动率策略](../volatility-hunter/) - 波动率狩猎

## 🔧 开发资源

### 代码质量

- 运行测试：`cargo test --workspace`
- 代码检查：`cargo clippy --workspace`
- 格式化：`cargo fmt --all`
- 安全审计：`cargo audit`

### 文档生成

```bash
# 生成 API 文档
cargo doc --workspace --no-deps

# 在浏览器中查看
open target/doc/index.html  # macOS
xdg-open target/doc/index.html  # Linux
```

## 📋 状态

| 文档 | 状态 | 最后更新 |
|------|------|---------|
| 错误处理指南 | ✅ 完成 | 2026-03-05 |
| 安全指南 | ✅ 完成 | 2026-03-05 |
| 依赖升级计划 | ✅ 完成 | 2026-03-05 |
| API 文档 | 📝 进行中 | - |
| 性能基准 | 📝 进行中 | - |
| 监控告警 | 📝 进行中 | - |

## 🤝 贡献

欢迎提交 Issue 和 Pull Request 来改进文档！

---

**维护者**: PolyMarket Knife Team  
**许可**: MIT
