#!/bin/bash
# 获取 Polymarket 活跃市场列表
# 用法：./fetch-active-markets.sh [数量]

LIMIT=${1:-20}

echo "📊 获取 Polymarket 活跃市场 (流动性 > 0)..."
echo "=============================================="

curl -s "https://gamma-api.polymarket.com/markets?limit=100&active=true&closed=false" | \
  jq -r --argjson limit "$LIMIT" '
    .[] | 
    select(.closed == false and .liquidityNum > 0) | 
    {
      token_id: (.clobTokenIds | split("[")[1] | split("]")[0] | split(",")[0] | gsub("\""; "")),
      title: .question,
      liquidity: .liquidityNum,
      volume: .volumeNum
    } | 
    "\(.token_id) | \(.title) | Liq: $\(.liquidity | tostring | split(".")[0]) | Vol: $\(.volume | tostring | split(".")[0])"
  ' 2>/dev/null | head -n $LIMIT

echo ""
echo "💡 使用方法:"
echo "1. 复制上面的 token_id (第一列)"
echo "2. 更新配置文件 config/market-maker-mainnet-test.toml"
echo "3. 修改 market_ids = [\"你的 token_id\"]"
