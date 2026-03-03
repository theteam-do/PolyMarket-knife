#!/bin/bash
# PolyMarket Knife - 本地编译并部署到 VPS
# 优势：本地编译快，节省 VPS 资源，模拟真实部署流程

set -e

# 配置
VPS_USER="${VPS_USER:-root}"
VPS_HOST="${VPS_HOST:-139.180.207.66}"
VPS_KEY="${VPS_KEY:-$HOME/works/agent-keys/agent}"
PROJECT_PATH="/root/works/PolyMarket-knife"
SSH_OPTS="-o IdentitiesOnly=yes -i $VPS_KEY -o ConnectTimeout=10"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m'

print_header() { echo -e "\n${MAGENTA}=== $1 ===${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_info() { echo -e "   $1"; }

# 检查 SSH 密钥
if [ ! -f "$VPS_KEY" ]; then
    print_error "SSH 密钥不存在：$VPS_KEY"
    exit 1
fi

print_header "PolyMarket Knife - 本地编译并部署到 VPS"
print_info "VPS: $VPS_USER@$VPS_HOST"
print_info "项目路径：$PROJECT_PATH"
echo ""

# 阶段 1: 本地编译
print_header "阶段 1: 本地编译"

print_info "清理旧构建..."
cargo clean

print_info "编译 release 版本..."
time cargo build --release

print_info "验证二进制文件..."
BINARIES=("market-maker" "arbitrage" "follow-trade" "volatility-hunter" "info-edge" "order-attack")
for binary in "${BINARIES[@]}"; do
    if [ -f "target/release/$binary" ]; then
        SIZE=$(ls -lh "target/release/$binary" | awk '{print $5}')
        print_success "$binary ($SIZE)"
    else
        print_error "$binary 编译失败"
        exit 1
    fi
done

# 阶段 2: 上传到 VPS
print_header "阶段 2: 上传到 VPS"

print_info "检查 VPS 连接..."
if ! ssh $SSH_OPTS -o BatchMode=yes "$VPS_USER@$VPS_HOST" "echo OK" > /dev/null 2>&1; then
    print_error "无法连接到 VPS"
    exit 1
fi
print_success "VPS 连接正常"

print_info "创建备份目录..."
ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
    mkdir -p $PROJECT_PATH/backup
    if [ -d '$PROJECT_PATH/target/release' ]; then
        cp -r $PROJECT_PATH/target/release $PROJECT_PATH/backup/release_$(date +%Y%m%d_%H%M%S)
        echo '备份完成'
    fi
"

print_info "上传二进制文件..."
scp $SSH_OPTS target/release/market-maker $VPS_USER@$VPS_HOST:$PROJECT_PATH/target/release/
scp $SSH_OPTS target/release/arbitrage $VPS_USER@$VPS_HOST:$PROJECT_PATH/target/release/
scp $SSH_OPTS target/release/follow-trade $VPS_USER@$VPS_HOST:$PROJECT_PATH/target/release/
scp $SSH_OPTS target/release/volatility-hunter $VPS_USER@$VPS_HOST:$PROJECT_PATH/target/release/
scp $SSH_OPTS target/release/info-edge $VPS_USER@$VPS_HOST:$PROJECT_PATH/target/release/
scp $SSH_OPTS target/release/order-attack $VPS_USER@$VPS_HOST:$PROJECT_PATH/target/release/

print_success "二进制文件上传完成"

print_info "上传测试配置文件..."
scp $SSH_OPTS config/*-mainnet-test.toml $VPS_USER@$VPS_HOST:$PROJECT_PATH/config/ 2>/dev/null || true
print_success "配置文件上传完成"

# 阶段 3: 验证
print_header "阶段 3: 验证"

print_info "验证 VPS 上的二进制文件..."
ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
    cd $PROJECT_PATH
    echo '=== 二进制文件 ==='
    ls -lh target/release/market-maker target/release/arbitrage target/release/follow-trade target/release/volatility-hunter
    
    echo ''
    echo '=== 配置文件 ==='
    ls -lh config/*-mainnet-test.toml 2>/dev/null || echo '未找到测试配置文件'
    
    echo ''
    echo '=== 测试运行 (market-maker --help) ==='
    ./target/release/market-maker --help | head -5
"

print_success "验证完成"

# 阶段 4: 运行测试
print_header "阶段 4: 运行测试"
echo ""
echo "是否立即运行主网测试？"
echo "1) 运行 Market Maker 测试 (120 秒)"
echo "2) 运行所有策略测试"
echo "3) 跳过，稍后手动运行"
read -p "选择 [1-3]: " run_choice

case $run_choice in
    1)
        print_info "运行 Market Maker 测试..."
        ./scripts/mainnet-test.sh
        ;;
    2)
        print_info "运行所有策略测试..."
        ./scripts/mainnet-test.sh
        ;;
    3)
        print_info "稍后可以运行：./scripts/mainnet-test.sh"
        ;;
esac

print_header "部署完成"
print_success "所有策略已部署到 VPS"
echo ""
print_info "下一步:"
print_info "1. 运行测试：./scripts/mainnet-test.sh"
print_info "2. 查看日志：ssh $VPS_USER@$VPS_HOST 'tail -f $PROJECT_PATH/logs/*.log'"
print_info "3. 监控钱包：https://polygonscan.com/address/0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6"
