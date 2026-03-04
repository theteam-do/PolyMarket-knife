# API 文档

本目录包含 PolyMarket Knife 项目的 API 文档和使用指南。

## 📚 Crate 文档

### 核心 Crate

| Crate | 文档 | 描述 |
|-------|------|------|
| [`common`](../common/) | [API Docs](../target/doc/common/) | 共享配置和工具 |
| [`monitor`](../monitor/) | [API Docs](../target/doc/monitor/) | 监控告警系统 |
| [`market-maker`](../market-maker/) | [API Docs](../target/doc/market_maker/) | 做市策略 |
| [`arbitrage`](../arbitrage/) | [API Docs](../target/doc/arbitrage/) | 套利策略 |
| [`follow-trade`](../follow-trade/) | [API Docs](../target/doc/follow_trade/) | 跟单策略 |

### 依赖 Crate

- [`polymarket-client-sdk`](../rs-clob-client/) - Polymarket CLOB API 客户端

## 🔧 开发指南

### 生成 API 文档

```bash
# 生成所有 crate 的文档
cargo doc --workspace --no-deps

# 生成并打开文档
cargo doc --workspace --no-deps --open

# 包含私有 API（开发用）
cargo doc --workspace --document-private-items
```

### 文档规范

#### 模块级文档

每个模块应该在文件开头包含模块级文档：

```rust
//! 模块描述
//!
//! # 功能
//!
//! - 功能 1
//! - 功能 2
//!
//! # 使用示例
//!
//! ```rust
//! // 示例代码
//! ```
```

#### 函数文档

公共函数应该包含完整的文档：

```rust
/// 创建新的订单簿
///
/// # 参数
///
/// * `token_id` - 市场 Token ID
///
/// # 返回
///
/// 返回初始化的订单簿，买单和卖单为空
///
/// # 示例
///
/// ```
/// use market_maker::order_book::OrderBook;
///
/// let book = OrderBook::new("123".to_string());
/// assert!(book.bids.is_empty());
/// assert!(book.asks.is_empty());
/// ```
pub fn new(token_id: String) -> Self {
    // ...
}
```

#### 错误文档

如果函数返回 `Result`，应该说明可能的错误：

```rust
/// 加载配置文件
///
/// # 错误
///
/// 如果文件不存在或格式错误，返回 `anyhow::Error`
///
/// # 示例
///
/// ```rust
/// # use anyhow::Result;
/// fn load_config() -> Result<Config> {
///     // ...
/// }
/// ```
```

## 📖 使用指南

### 快速开始

1. [快速开始指南](../QUICKSTART.md)
2. [配置说明](../config/)
3. [错误处理](ERROR_HANDLING.md)

### 策略文档

- [做市策略](../market-maker/)
- [套利策略](../arbitrage/)
- [跟单策略](../follow-trade/)

### 性能优化

- [性能基准测试](BENCHMARKING.md)
- [性能分析工具](#性能分析)

## 🔍 代码搜索

### 查找符号

```bash
# 使用 rust-analyzer
# 在 VSCode 中按 F12 跳转到定义

# 使用命令行
cargo doc --open
# 在浏览器中搜索
```

### 查找使用位置

```bash
# 使用 ripgrep
rg "fn place_order" --type rust

# 使用 cargo-geiger（检查 unsafe 代码）
cargo install cargo-geiger
cargo geiger
```

## 📊 质量指标

### 文档覆盖率

```bash
# 检查文档覆盖率
cargo doc --workspace --no-deps 2>&1 | grep -i "undocumented"
```

### 测试覆盖率

```bash
# 安装 cargo-tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --workspace --out Html
```

## 🤝 贡献

### 提交文档

1. 确保所有公共 API 都有文档
2. 添加使用示例
3. 运行 `cargo doc` 验证
4. 提交 PR

### 文档审查清单

- [ ] 所有公共函数有文档
- [ ] 包含使用示例
- [ ] 错误情况说明
- [ ] 代码示例可编译
- [ ] 无语法错误

---

**最后更新**: 2026-03-05  
**维护者**: PolyMarket Knife Team
