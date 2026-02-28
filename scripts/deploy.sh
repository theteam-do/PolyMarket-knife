#!/bin/bash
# 部署到 VPS

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <vps_host>"
    exit 1
fi

VPS_HOST=$1

echo "🚀 Deploying to $VPS_HOST..."

# 编译
./build-all.sh

# 创建部署包
DEPLOY_DIR="/tmp/polymarket-knife-deploy"
rm -rf "$DEPLOY_DIR"
mkdir -p "$DEPLOY_DIR"

# 复制二进制文件
for dir in market-maker arbitrage follow-trade volatility-hunter info-edge order-attack; do
    cp "$dir/target/release/$dir" "$DEPLOY_DIR/"
done

# 复制配置
cp -r config "$DEPLOY_DIR/"

# 复制到 VPS
scp -r "$DEPLOY_DIR"/* "de@$VPS_HOST:~/polymarket-knife/"

echo "✅ Deployment complete!"
echo ""
echo "On VPS, run:"
echo "  cd ~/polymarket-knife"
echo "  ./market-maker --config config/market-maker.toml &"
