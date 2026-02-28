#!/bin/bash
# 测试网验证脚本

set -e

echo "======================================"
echo "  PolyMarket Testnet Verification"
echo "======================================"

# 检查私钥
if [ -z "$POLYMARKET_PRIVATE_KEY" ]; then
    echo "❌ Error: POLYMARKET_PRIVATE_KEY not set"
    echo "   Please run: export POLYMARKET_PRIVATE_KEY='your_key'"
    exit 1
fi

echo "✅ Private key configured"

# 检查配置文件
CONFIG_FILE="${1:-config/market-maker-testnet.toml}"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Error: Config file not found: $CONFIG_FILE"
    exit 1
fi

echo "✅ Config file: $CONFIG_FILE"

# 检查二进制文件
BINARY="./target/release/market-maker"
if [ ! -f "$BINARY" ]; then
    echo "❌ Error: Binary not found: $BINARY"
    echo "   Please run: cargo build --release"
    exit 1
fi

echo "✅ Binary found: $BINARY"

# 运行测试
echo ""
echo "Starting test..."
echo "--------------------------------------"

timeout 60 $BINARY --config $CONFIG_FILE || {
    if [ $? -eq 124 ]; then
        echo "--------------------------------------"
        echo "✅ Test completed (60s timeout)"
    else
        echo "--------------------------------------"
        echo "❌ Test failed"
        exit 1
    fi
}

echo ""
echo "======================================"
echo "  Test Summary"
echo "======================================"
echo "✅ All checks passed!"
echo ""
echo "Next steps:"
echo "1. Check logs for errors"
echo "2. Verify orders on testnet"
echo "3. Adjust parameters if needed"
