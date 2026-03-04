# 性能基准测试指南

本文档介绍如何运行和解读 PolyMarket Knife 项目的性能基准测试。

---

## 运行基准测试

### 运行所有基准测试

```bash
cargo bench --workspace
```

### 运行特定 crate 的基准测试

```bash
# Market Maker 基准测试
cargo bench -p market-maker

# Common 基准测试
cargo bench -p common
```

### 运行特定基准测试

```bash
# 只运行订单簿相关基准
cargo bench -p market-maker --bench performance -- orderbook

# 只运行 PnL 计算基准
cargo bench -p market-maker --bench performance -- pnl
```

---

## 基准测试位置

### Market Maker

- **文件**: `market-maker/benches/performance.rs`
- **测试项目**:
  - 订单簿中间价计算
  - Decimal vs f64 性能对比
  - 报价计算
  - PnL 计算
  - 风控检查

---

## 解读结果

### 示例输出

```
running 5 tests
test orderbook_mid_price          ... bench:          0.5 ns/iter (+/- 0.1)
test decimal_vs_f64/f64_addition  ... bench:          0.3 ns/iter (+/- 0.0)
test decimal_vs_f64/decimal_addition ... bench:       5.2 ns/iter (+/- 0.3)
test quote_calculation/spread_bps_100 ... bench:      1.2 ns/iter (+/- 0.1)
test pnl_calculation/simple_pnl   ... bench:          8.5 ns/iter (+/- 0.5)
```

### 指标说明

- **ns/iter**: 每次迭代的纳秒数（越小越好）
- **+/-**: 标准差（表示稳定性）
- **iter**: 测试迭代次数

### 性能对比

| 操作 | f64 | Decimal | 开销倍数 |
|------|-----|---------|----------|
| 加法 | 0.3ns | 5.2ns | ~17x |
| 乘法 | 0.4ns | 6.1ns | ~15x |
| 比较 | 0.2ns | 4.8ns | ~24x |

**注意**: Decimal 虽然慢，但提供精确的十进制计算，适合金融场景。

---

## 性能优化建议

### 1. 热点代码使用 f64

```rust
// ✅ 推荐：中间计算使用 f64
let mid_price_f64 = (bid + ask) / 2.0;

// ❌ 避免：频繁创建 Decimal
let mid_price = (bid_dec + ask_dec) / dec!(2);
```

### 2. 边界处转换

```rust
// ✅ 推荐：只在输入输出使用 Decimal
fn calculate_pnl(entry: Decimal, exit: Decimal, shares: Decimal) -> Decimal {
    // 内部计算可以使用 f64
    let entry_f64 = entry.to_f64().unwrap();
    let exit_f64 = exit.to_f64().unwrap();
    let shares_f64 = shares.to_f64().unwrap();
    
    let pnl_f64 = (exit_f64 - entry_f64) * shares_f64;
    Decimal::from_f64_retain(pnl_f64).unwrap()
}
```

### 3. 预分配容量

```rust
// ✅ 推荐：预分配 Vec 容量
let mut orders = Vec::with_capacity(100);

// ❌ 避免：多次重新分配
let mut orders = Vec::new();
```

---

## 性能回归检测

### CI/CD 集成

```yaml
# .github/workflows/bench.yml
name: Benchmark

on: [push]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench --workspace
      - name: Store results
        uses: benchmark-action/github-action-benchmark@v1
```

### 阈值告警

如果基准测试性能下降超过 10%，应该：

1. 审查最近的代码变更
2. 分析性能瓶颈（使用 `perf` 或 `flamegraph`）
3. 优化热点代码
4. 重新运行基准测试验证

---

## 工具推荐

### 性能分析

```bash
# 安装 flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bin market-maker

# 查看 CPU 热点
perf record -g ./target/release/market-maker
perf report
```

### 内存分析

```bash
# 安装 cargo-massif
cargo install cargo-massif

# 分析内存使用
cargo massif -p market-maker
```

---

## 参考资源

- [Criterion 文档](https://bheisler.github.io/criterion.rs/book/)
- [Rust 性能优化指南](https://github.com/pretzelhammer/rust-blog/blob/master/posts/fast-rust-programming.md)
- [flamegraph](https://github.com/flamegraph-rs/flamegraph)

---

**最后更新**: 2026-03-05  
**维护者**: PolyMarket Knife Team
