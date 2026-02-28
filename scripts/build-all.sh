#!/bin/bash
# 编译所有策略程序

set -e

echo "🔨 Building all PolyMarket Knife strategies..."

cd "$(dirname "$0")/.."

# 编译所有程序
for dir in market-maker arbitrage follow-trade volatility-hunter info-edge order-attack; do
    echo "Building $dir..."
    cd "$dir"
    cargo build --release
    cd ..
done

echo "✅ All strategies built successfully!"
echo ""
echo "Binaries location:"
for dir in market-maker arbitrage follow-trade volatility-hunter info-edge order-attack; do
    echo "  - $dir/target/release/$dir"
done
