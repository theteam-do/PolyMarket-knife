#!/bin/bash
# PolyMarket Knife VPS 快速连接脚本
# 快速 SSH 连接到目标服务器

set -e

# 配置
VPS_USER="${VPS_USER:-root}"
VPS_HOST="${VPS_HOST:-95.179.239.239}"
VPS_KEY="${VPS_KEY:-$HOME/works/agent-keys/agent}"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}$1${NC}"
}

print_error() {
    echo -e "${RED}$1${NC}"
}

# 检查密钥
if [ ! -f "$VPS_KEY" ]; then
    print_error "SSH 密钥不存在：$VPS_KEY"
    exit 1
fi

print_info "======================================"
print_info "  PolyMarket Knife VPS 连接"
print_info "======================================"
print_info "主机：$VPS_HOST"
print_info "用户：$VPS_USER"
print_info "密钥：$VPS_KEY"
print_info "======================================"

# 测试连接
print_info "测试连接..."
if ssh -o ConnectTimeout=10 -o BatchMode=yes \
     -o IdentitiesOnly=yes -i "$VPS_KEY" \
     "$VPS_USER@$VPS_HOST" "echo OK" > /dev/null 2>&1; then
    print_success "连接成功!"
else
    print_error "连接失败"
    exit 1
fi

# 连接
print_info "正在连接..."
ssh -o IdentitiesOnly=yes -i "$VPS_KEY" "$VPS_USER@$VPS_HOST" "$@"
