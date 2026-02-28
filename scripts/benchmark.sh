#!/bin/bash
# 性能基准测试脚本

set -e

echo "======================================"
echo "  PolyMarket Knife Benchmark"
echo "======================================"

# 编译 release 版本
echo "Building release version..."
cargo build --release

echo ""
echo "Running benchmarks..."
echo "--------------------------------------"

# 运行基准测试
cargo bench --all 2>&1 | tee benchmark-results.txt

echo ""
echo "======================================"
echo "  Benchmark Complete"
echo "======================================"
echo "Results saved to: benchmark-results.txt"
echo ""
echo "View detailed results:"
echo "  open target/criterion/*/report/index.html"
