#!/bin/bash
# 主网部署验证脚本
# 用法：./scripts/verify-mainnet.sh [策略名称]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

STRATEGY=${1:-all}

echo "========================================"
echo "  主网部署验证脚本"
echo "========================================"
echo ""

# 检查项计数
PASSED=0
FAILED=0
WARNINGS=0

# 验证函数
check_pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARNINGS++))
}

# 1. 检查 Rust 环境
echo "1. 检查 Rust 环境..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    if [[ $(echo $RUST_VERSION | cut -d. -f1) -ge 1 ]] && [[ $(echo $RUST_VERSION | cut -d. -f2) -ge 75 ]]; then
        check_pass "Rust 版本：$RUST_VERSION"
    else
        check_fail "Rust 版本过低：$RUST_VERSION (需要 >= 1.75)"
    fi
else
    check_fail "Rust 未安装"
fi

# 2. 检查编译
echo ""
echo "2. 检查编译..."
if cargo build --release --quiet 2>&1 | grep -q "error"; then
    check_fail "编译失败"
else
    check_pass "编译成功"
fi

# 3. 检查测试
echo ""
echo "3. 检查测试..."
if cargo test --release --quiet 2>&1 | grep -q "test result: ok"; then
    check_pass "测试通过"
else
    check_warn "测试未运行或失败"
fi

# 4. 检查私钥环境变量
echo ""
echo "4. 检查私钥配置..."
if [ -n "$POLYMARKET_PRIVATE_KEY" ]; then
    KEY_LENGTH=${#POLYMARKET_PRIVATE_KEY}
    if [ $KEY_LENGTH -eq 66 ] || [ $KEY_LENGTH -eq 64 ]; then
        check_pass "私钥已设置 (长度：$KEY_LENGTH)"
    else
        check_warn "私钥长度异常：$KEY_LENGTH (应该是 64 或 66)"
    fi
else
    check_fail "私钥未设置 (POLYMARKET_PRIVATE_KEY)"
    echo "   请运行：export POLYMARKET_PRIVATE_KEY=\"your_private_key\""
fi

# 5. 检查配置文件
echo ""
echo "5. 检查配置文件..."

check_config() {
    local config_file=$1
    local strategy_name=$2
    
    if [ ! -f "$config_file" ]; then
        check_warn "配置文件不存在：$config_file"
        return
    fi
    
    # 检查 mode
    if grep -q 'mode = "live"' "$config_file"; then
        check_pass "$strategy_name: mode=live"
    else
        check_warn "$strategy_name: mode 不是 live"
    fi
    
    # 检查 environment
    if grep -q 'environment = "mainnet"' "$config_file"; then
        check_pass "$strategy_name: environment=mainnet"
    else
        check_warn "$strategy_name: environment 不是 mainnet"
    fi
    
    # 检查 live_acknowledged
    if grep -q 'live_acknowledged = true' "$config_file"; then
        check_pass "$strategy_name: live_acknowledged=true"
    else
        check_warn "$strategy_name: live_acknowledged 不是 true"
    fi
    
    # 检查是否有硬编码私钥
    if grep -q 'private_key = "0x' "$config_file"; then
        check_fail "$strategy_name: 配置文件中包含私钥 (安全风险!)"
    else
        check_pass "$strategy_name: 未硬编码私钥"
    fi
}

if [ "$STRATEGY" == "all" ] || [ "$STRATEGY" == "arbitrage" ]; then
    echo ""
    echo "Arbitrage 配置:"
    check_config "config/arbitrage-mainnet.toml" "Arbitrage"
fi

if [ "$STRATEGY" == "all" ] || [ "$STRATEGY" == "follow-trade" ]; then
    echo ""
    echo "Follow Trade 配置:"
    check_config "config/follow-trade-mainnet.toml" "Follow Trade"
fi

if [ "$STRATEGY" == "all" ] || [ "$STRATEGY" == "market-maker" ]; then
    echo ""
    echo "Market Maker 配置:"
    check_config "config/market-maker-mainnet.toml" "Market Maker"
fi

# 6. 检查文件权限
echo ""
echo "6. 检查文件权限..."
for config in config/*.toml; do
    if [ -f "$config" ]; then
        PERMS=$(stat -c %a "$config" 2>/dev/null || stat -f %Lp "$config" 2>/dev/null)
        if [ "$PERMS" == "600" ] || [ "$PERMS" == "400" ]; then
            check_pass "$config 权限：$PERMS"
        else
            check_warn "$config 权限：$PERMS (建议 600)"
        fi
    fi
done

# 7. 检查网络连通性
echo ""
echo "7. 检查网络连通性..."

# 检查 Polygon RPC
if curl -s --max-time 5 https://polygon-rpc.com > /dev/null; then
    check_pass "Polygon RPC 可达"
else
    check_fail "Polygon RPC 不可达"
fi

# 检查 Polymarket CLOB
if curl -s --max-time 5 https://clob.polymarket.com > /dev/null; then
    check_pass "Polymarket CLOB 可达"
else
    check_fail "Polymarket CLOB 不可达"
fi

# 8. 检查二进制文件
echo ""
echo "8. 检查二进制文件..."
for strategy in arbitrage follow-trade market-maker volatility-hunter info-edge order-attack; do
    if [ -x "target/release/$strategy" ]; then
        check_pass "$strategy 二进制文件存在"
    else
        check_fail "$strategy 二进制文件不存在"
    fi
done

# 9. 检查主网配置参数
echo ""
echo "9. 检查主网配置参数..."

check_risk_params() {
    local config_file=$1
    local strategy_name=$2
    
    if [ ! -f "$config_file" ]; then
        return
    fi
    
    # 检查 max_loss_per_day 或类似参数
    if grep -q "max_loss_per_day\|max_daily_loss" "$config_file"; then
        check_pass "$strategy_name: 已设置日亏损限制"
    else
        check_warn "$strategy_name: 未设置日亏损限制"
    fi
    
    # 检查 max_position
    if grep -q "max_position" "$config_file"; then
        check_pass "$strategy_name: 已设置持仓限制"
    else
        check_warn "$strategy_name: 未设置持仓限制"
    fi
}

if [ "$STRATEGY" == "all" ] || [ "$STRATEGY" == "arbitrage" ]; then
    check_risk_params "config/arbitrage-mainnet.toml" "Arbitrage"
fi

if [ "$STRATEGY" == "all" ] || [ "$STRATEGY" == "follow-trade" ]; then
    check_risk_params "config/follow-trade-mainnet.toml" "Follow Trade"
fi

if [ "$STRATEGY" == "all" ] || [ "$STRATEGY" == "market-maker" ]; then
    check_risk_params "config/market-maker-mainnet.toml" "Market Maker"
fi

# 总结
echo ""
echo "========================================"
echo "  验证结果"
echo "========================================"
echo -e "${GREEN}通过${NC}: $PASSED"
echo -e "${RED}失败${NC}: $FAILED"
echo -e "${YELLOW}警告${NC}: $WARNINGS"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}✗ 验证失败${NC} - 请修复上述问题后再部署主网"
    exit 1
elif [ $WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}⚠ 验证通过但有警告${NC} - 请检查警告项后谨慎部署"
    exit 0
else
    echo -e "${GREEN}✓ 验证通过${NC} - 可以部署主网"
    exit 0
fi
