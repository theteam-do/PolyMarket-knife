#!/bin/bash
# PolyMarket Knife VPS 调试脚本
# 自动化诊断 VPS 连接、环境和运行状态

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
VPS_USER="${VPS_USER:-root}"
VPS_HOST="${VPS_HOST:-95.179.239.239}"
VPS_KEY="${VPS_KEY:-$HOME/works/agent-keys/agent}"
PROJECT_PATH="/home/de/works/PolyMarket-knife"
LOG_DIR="./vps-debug-logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# SSH 选项
SSH_OPTS="-o IdentitiesOnly=yes -i $VPS_KEY -o ConnectTimeout=10 -o StrictHostKeyChecking=no"

# Polymarket 测试钱包配置 (主网)
POLY_ADDRESS="0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6"
POLY_PRIVATE_KEY="0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7"

# 打印函数
print_header() {
    echo -e "\n${BLUE}======================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}======================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "   $1"
}

# 检查参数
check_params() {
    if [ -z "$VPS_HOST" ]; then
        print_error "VPS_HOST 未设置"
        echo ""
        echo "请设置环境变量:"
        echo "  export VPS_HOST='your_vps_ip'"
        echo "  export VPS_USER='your_username' (可选，默认：de)"
        echo ""
        echo "或者使用命令行参数:"
        echo "  $0 <vps_ip> [username]"
        exit 1
    fi
}

# 创建日志目录
setup_log_dir() {
    mkdir -p "$LOG_DIR"
    print_info "日志目录：$LOG_DIR"
}

# SSH 连接测试
test_ssh_connection() {
    print_header "阶段 1: SSH 连接测试"
    
    # 检查密钥文件
    if [ ! -f "$VPS_KEY" ]; then
        print_error "SSH 密钥不存在：$VPS_KEY"
        return 1
    fi
    print_success "SSH 密钥已找到：$VPS_KEY"
    
    # 测试基本连接
    print_info "测试 SSH 连接..."
    if ssh $SSH_OPTS -o BatchMode=yes "$VPS_USER@$VPS_HOST" "echo OK" > /dev/null 2>&1; then
        print_success "SSH 连接正常"
    else
        print_error "SSH 连接失败"
        print_info "尝试诊断..."
        
        # 检查端口
        print_info "检查端口 22..."
        if nc -zv -w5 "$VPS_HOST" 22 2>&1 | grep -q "succeeded"; then
            print_success "端口 22 开放"
        else
            print_error "端口 22 无法访问"
        fi
        
        # 检查网络延迟
        print_info "测试网络延迟..."
        if command -v ping > /dev/null 2>&1; then
            ping -c 3 "$VPS_HOST" 2>/dev/null | tail -1 || print_warning "无法 ping 通 VPS"
        fi
        
        return 1
    fi
    
    # 获取连接信息
    print_info "获取连接详情..."
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        echo \"主机名：\$(hostname)\"
        echo \"内核版本：\$(uname -r)\"
        echo \"运行时间：\$(uptime -p 2>/dev/null || uptime)\"
    " 2>/dev/null
}

# 系统资源检查
check_system_resources() {
    print_header "阶段 2: 系统资源检查"
    
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        echo '=== CPU 信息 ==='
        lscpu | grep -E 'Model name|CPU\\(s\\)|Thread' || echo '无法获取 CPU 信息'
        
        echo ''
        echo '=== 内存使用 ==='
        free -h
        
        echo ''
        echo '=== 磁盘空间 ==='
        df -h /home
        
        echo ''
        echo '=== 系统负载 ==='
        uptime
    " 2>/dev/null
    
    # 检查关键资源阈值
    MEMORY_AVAIL=$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "free -m | awk 'NR==2{print \$7}'" 2>/dev/null)
    if [ -n "$MEMORY_AVAIL" ] && [ "$MEMORY_AVAIL" -lt 500 ]; then
        print_warning "可用内存不足 500MB (当前：${MEMORY_AVAIL}MB)"
    else
        print_success "内存充足"
    fi
    
    DISK_AVAIL=$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "df -h /home | awk 'NR==2{print \$4}'" 2>/dev/null | sed 's/G//')
    if [ -n "$DISK_AVAIL" ] && [ "${DISK_AVAIL%.*}" -lt 5 ]; then
        print_warning "磁盘空间不足 5GB (当前：${DISK_AVAIL}G)"
    else
        print_success "磁盘空间充足"
    fi
}

# Rust 环境检查
check_rust_environment() {
    print_header "阶段 3: Rust 环境检查"
    
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        echo '=== Rust 版本 ==='
        rustc --version 2>/dev/null || echo 'Rust 未安装'
        
        echo ''
        echo '=== Cargo 版本 ==='
        cargo --version 2>/dev/null || echo 'Cargo 未安装'
        
        echo ''
        echo '=== Rust 目标平台 ==='
        rustc -vV 2>/dev/null | grep host || true
        
        echo ''
        echo '=== Cargo 缓存大小 ==='
        du -sh ~/.cargo/registry 2>/dev/null || echo '无法获取缓存大小'
    " 2>/dev/null
    
    # 检查 Rust 版本
    RUST_VERSION=$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "rustc --version 2>/dev/null | awk '{print \$2}'" 2>/dev/null)
    if [ -n "$RUST_VERSION" ]; then
        print_success "Rust 版本：$RUST_VERSION"
        
        # 检查版本是否 >= 1.75
        RUST_MINOR=$(echo "$RUST_VERSION" | cut -d. -f2)
        if [ "${RUST_MINOR:-0}" -lt 75 ]; then
            print_warning "Rust 版本可能过旧 (推荐 >= 1.75)"
        fi
    else
        print_error "Rust 未安装"
        print_info "安装命令：curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi
}

# 项目状态检查
check_project_status() {
    print_header "阶段 4: 项目状态检查"
    
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        echo '=== 项目目录 ==='
        ls -la \"$PROJECT_PATH\" 2>/dev/null || echo '项目目录不存在'
        
        echo ''
        echo '=== Git 状态 ==='
        cd \"$PROJECT_PATH\" 2>/dev/null && git status --short 2>/dev/null || echo '无法获取 Git 状态'
        
        echo ''
        echo '=== 最近提交 ==='
        cd \"$PROJECT_PATH\" 2>/dev/null && git log -3 --oneline 2>/dev/null || true
        
        echo ''
        echo '=== 编译产物 ==='
        ls -lh \"$PROJECT_PATH/target/release/\" 2>/dev/null | grep -E 'market-maker|arbitrage|follow-trade' || echo '未找到编译产物'
    " 2>/dev/null
    
    # 检查二进制文件
    BINARY_EXISTS=$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "test -f '$PROJECT_PATH/target/release/market-maker' && echo 'yes' || echo 'no'" 2>/dev/null)
    if [ "$BINARY_EXISTS" = "yes" ]; then
        print_success "已编译二进制文件"
    else
        print_warning "未找到编译产物，需要重新编译"
    fi
}

# 配置文件检查
check_config_files() {
    print_header "阶段 5: 配置文件检查"
    
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        echo '=== 配置文件列表 ==='
        ls -la \"$PROJECT_PATH/config/\" 2>/dev/null
        
        echo ''
        echo '=== 环境变量文件 ==='
        ls -la \"$PROJECT_PATH/.env*\" 2>/dev/null || echo '未找到 .env 文件'
        
        echo ''
        echo '=== 环境变量检查 ==='
        env | grep -i polymarket || echo '未设置 POLYMARKET 相关环境变量'
    " 2>/dev/null
    
    # 检查私钥配置
    PRIVATE_KEY_SET=$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "test -n \"\$POLYMARKET_PRIVATE_KEY\" && echo 'yes' || echo 'no'" 2>/dev/null)
    if [ "$PRIVATE_KEY_SET" = "yes" ]; then
        print_success "POLYMARKET_PRIVATE_KEY 已设置"
    else
        print_warning "POLYMARKET_PRIVATE_KEY 未设置"
        print_info "设置方法：export POLYMARKET_PRIVATE_KEY='your_key'"
    fi
}

# 钱包余额检查
check_wallet_balance() {
    print_header "阶段 6: 钱包余额检查"
    
    print_info "钱包地址：$POLY_ADDRESS"
    
    # 检查 MATIC 余额
    print_info "查询 MATIC 余额..."
    MATIC_BALANCE=$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        node -e \"
        const { ethers } = require('ethers');
        const provider = new ethers.providers.JsonRpcProvider('https://polygon-bor-rpc.publicnode.com');
        provider.getBalance('$POLY_ADDRESS').then(b => console.log(ethers.utils.formatEther(b)));
        \" 2>/dev/null || echo '0'
    " 2>/dev/null)
    
    if [ -n "$MATIC_BALANCE" ] && [ "$MATIC_BALANCE" != "0" ]; then
        print_success "MATIC 余额：$MATIC_BALANCE"
        
        # 检查是否足够 gas
        BALANCE_NUM=$(echo "$MATIC_BALANCE" | awk '{printf "%.4f", $1}')
        if (( $(echo "$BALANCE_NUM < 0.1" | bc -l 2>/dev/null || echo "0") )); then
            print_warning "MATIC 余额不足 0.1，可能无法支付 gas"
        fi
    else
        print_warning "无法查询余额或余额为 0"
    fi
    
    # 检查 USDC 余额
    print_info "查询 USDC 余额..."
    USDC_BALANCE=$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        node -e \"
        const { ethers } = require('ethers');
        const provider = new ethers.providers.JsonRpcProvider('https://polygon-bor-rpc.publicnode.com');
        const usdcContract = '0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174';
        const abi = ['function balanceOf(address) view returns (uint256)'];
        const contract = new ethers.Contract(usdcContract, abi, provider);
        contract.balanceOf('$POLY_ADDRESS').then(b => console.log((b / 1e6).toFixed(2)));
        \" 2>/dev/null || echo '0'
    " 2>/dev/null)
    
    if [ -n "$USDC_BALANCE" ] && [ "$USDC_BALANCE" != "0" ]; then
        print_success "USDC 余额：$USDC_BALANCE"
    else
        print_warning "USDC 余额为 0 或无法查询"
    fi
}

# 主网测试
test_mainnet_strategies() {
    print_header "阶段 7: 主网策略测试 (最小订单)"
    
    print_warning "此阶段将在主网执行真实交易（最大 1 USDC）"
    print_info "钱包地址：$POLY_ADDRESS"
    echo ""
    read -p "确认继续？(输入 yes 继续): " -r
    if [[ $REPLY != "yes" ]]; then
        print_info "跳过主网测试"
        return 0
    fi
    
    # 选择测试的策略
    echo ""
    echo "选择要测试的策略:"
    echo "1) Market Maker"
    echo "2) Arbitrage"
    echo "3) Follow Trade"
    echo "4) Volatility Hunter"
    echo "5) 全部测试"
    echo "6) 跳过"
    read -p "选择 [1-6]: " -r strategy_choice
    
    case $strategy_choice in
        1) strategies=("market-maker") ;;
        2) strategies=("arbitrage") ;;
        3) strategies=("follow-trade") ;;
        4) strategies=("volatility-hunter") ;;
        5) strategies=("market-maker" "arbitrage" "follow-trade" "volatility-hunter") ;;
        *) print_info "跳过主网测试"; return 0 ;;
    esac
    
    for strategy in "${strategies[@]}"; do
        print_header "测试：$strategy"
        
        # 设置测试超时时间（秒）
        TEST_DURATION=120
        
        print_info "运行 $strategy 测试 (${TEST_DURATION}秒)..."
        print_info "日志文件：$LOG_DIR/${strategy}_mainnet_$TIMESTAMP.log"
        
        # 在 VPS 上运行测试
        ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
            cd \"$PROJECT_PATH\"
            
            # 设置环境变量
            export POLYMARKET_PRIVATE_KEY=\"$POLY_PRIVATE_KEY\"
            export POLYMARKET_ADDRESS=\"$POLY_ADDRESS\"
            export RUST_LOG=info
            
            # 选择配置文件
            CONFIG_FILE=\"config/${strategy}-mainnet.toml\"
            if [ ! -f \"\$CONFIG_FILE\" ]; then
                CONFIG_FILE=\"config/${strategy}.toml\"
            fi
            
            echo \"使用配置文件：\$CONFIG_FILE\"
            
            # 运行测试
            timeout $TEST_DURATION ./target/release/$strategy --config \$CONFIG_FILE 2>&1 | tee /tmp/${strategy}_mainnet_$TIMESTAMP.log
            
            # 检查退出码
            EXIT_CODE=\$?
            if [ \$EXIT_CODE -eq 124 ]; then
                echo '测试完成 (超时正常退出)'
            elif [ \$EXIT_CODE -eq 0 ]; then
                echo '测试正常完成'
            else
                echo \"测试异常退出 (代码：\$EXIT_CODE)\"
            fi
        " 2>/dev/null
        
        # 下载日志
        ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "cat /tmp/${strategy}_mainnet_$TIMESTAMP.log" > "$LOG_DIR/${strategy}_mainnet_$TIMESTAMP.log" 2>/dev/null || true
        
        # 分析日志
        print_info "分析日志..."
        ORDER_COUNT=$(grep -c -i "order.*created\|order.*filled" "$LOG_DIR/${strategy}_mainnet_$TIMESTAMP.log" 2>/dev/null || echo "0")
        ERROR_COUNT=$(grep -c -E 'ERROR|panic|failed' "$LOG_DIR/${strategy}_mainnet_$TIMESTAMP.log" 2>/dev/null || echo "0")
        
        if [ "$ORDER_COUNT" -gt 0 ]; then
            print_success "检测到 $ORDER_COUNT 个订单相关事件"
        fi
        
        if [ "$ERROR_COUNT" -gt 0 ]; then
            print_warning "发现 $ERROR_COUNT 个错误/警告"
            print_info "错误摘要:"
            grep -E 'ERROR|panic|failed' "$LOG_DIR/${strategy}_mainnet_$TIMESTAMP.log" | head -5
        else
            print_success "未发现明显错误"
        fi
        
        # 询问是否继续下一个
        if [ "${#strategies[@]}" -gt 1 ]; then
            read -p "继续测试下一个策略？(y/N): " -n 1 -r
            echo ""
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                break
            fi
        fi
    done
    
    print_info "主网测试日志已保存：$LOG_DIR/"
}

# 编译测试
test_compilation() {
    print_header "阶段 8: 编译测试"
    
    print_info "开始编译 (这可能需要几分钟)..."
    
    # 在 VPS 上编译
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        cd \"$PROJECT_PATH\"
        
        echo '=== 清理旧构建 ==='
        cargo clean
        
        echo ''
        echo '=== 更新依赖 ==='
        cargo update
        
        echo ''
        echo '=== 开始编译 ==='
        time cargo build --release 2>&1 | tee /tmp/build_$TIMESTAMP.log
        
        echo ''
        echo '=== 编译结果 ==='
        if [ \$? -eq 0 ]; then
            echo '编译成功!'
            ls -lh target/release/
        else
            echo '编译失败!'
            exit 1
        fi
    " 2>/dev/null
    
    # 下载编译日志
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "cat /tmp/build_$TIMESTAMP.log" > "$LOG_DIR/build_$TIMESTAMP.log" 2>/dev/null || true
    print_info "编译日志已保存：$LOG_DIR/build_$TIMESTAMP.log"
}

# 运行测试
test_runtime() {
    print_header "阶段 9: 运行测试"
    
    print_info "启动程序测试 (60 秒超时)..."
    
    # 在 VPS 上运行测试
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "
        cd \"$PROJECT_PATH\"
        
        # 设置日志
        export RUST_LOG=info
        
        # 运行 60 秒
        timeout 60 ./target/release/market-maker --config config/market-maker.toml 2>&1 | tee /tmp/runtime_$TIMESTAMP.log || {
            if [ \$? -eq 124 ]; then
                echo '测试完成 (60s 超时)'
            else
                echo '程序运行出错'
            fi
        }
    " 2>/dev/null
    
    # 下载运行日志
    ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "cat /tmp/runtime_$TIMESTAMP.log" > "$LOG_DIR/runtime_$TIMESTAMP.log" 2>/dev/null || true
    print_info "运行日志已保存：$LOG_DIR/runtime_$TIMESTAMP.log"
    
    # 检查错误
    ERROR_COUNT=$(grep -c -E 'ERROR|panic|failed' "$LOG_DIR/runtime_$TIMESTAMP.log" 2>/dev/null || echo "0")
    if [ "$ERROR_COUNT" -gt 0 ]; then
        print_warning "发现 $ERROR_COUNT 个错误/警告"
        print_info "错误摘要:"
        grep -E 'ERROR|panic|failed' "$LOG_DIR/runtime_$TIMESTAMP.log" | head -10
    else
        print_success "未发现明显错误"
    fi
}

# 生成诊断报告
generate_report() {
    print_header "生成诊断报告"
    
    REPORT_FILE="$LOG_DIR/diagnostic_report_$TIMESTAMP.md"
    
    cat > "$REPORT_FILE" << EOF
# VPS 调试诊断报告

**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')
**VPS 主机**: $VPS_HOST
**用户**: $VPS_USER
**SSH 密钥**: $VPS_KEY
**测试钱包**: $POLY_ADDRESS

## 连接命令
\`\`\`bash
ssh root@$VPS_HOST -o IdentitiesOnly=yes -i $VPS_KEY
\`\`\`

## 检查摘要

| 检查项 | 状态 |
|--------|------|
| SSH 连接 | $(ssh $SSH_OPTS -o BatchMode=yes "$VPS_USER@$VPS_HOST" "echo OK" > /dev/null 2>&1 && echo "✅ 正常" || echo "❌ 失败") |
| Rust 环境 | $(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "rustc --version" > /dev/null 2>&1 && echo "✅ 已安装" || echo "❌ 未安装") |
| 项目编译 | $(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "test -f '$PROJECT_PATH/target/release/market-maker'" > /dev/null 2>&1 && echo "✅ 已编译" || echo "❌ 未编译") |
| 配置文件 | $(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "test -d '$PROJECT_PATH/config'" > /dev/null 2>&1 && echo "✅ 存在" || echo "❌ 缺失") |
| 钱包配置 | $(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "test -n \"\$POLYMARKET_PRIVATE_KEY\"" > /dev/null 2>&1 && echo "✅ 已设置" || echo "❌ 未设置") |

## 系统资源

\`\`\`
$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "free -h && echo '' && df -h /home" 2>/dev/null)
\`\`\`

## Rust 环境

\`\`\`
$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "rustc --version && cargo --version" 2>/dev/null)
\`\`\`

## 钱包余额

| 代币 | 余额 |
|------|------|
| MATIC | $(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "node -e \"const { ethers } = require('ethers'); const provider = new ethers.providers.JsonRpcProvider('https://polygon-bor-rpc.publicnode.com'); provider.getBalance('$POLY_ADDRESS').then(b => console.log(ethers.utils.formatEther(b))).catch(() => console.log('N/A'))\"" 2>/dev/null || echo "N/A") |
| USDC | $(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "node -e \"const { ethers } = require('ethers'); const provider = new ethers.providers.JsonRpcProvider('https://polygon-bor-rpc.publicnode.com'); const abi = ['function balanceOf(address) view returns (uint256)']; const contract = new ethers.Contract('0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174', abi, provider); contract.balanceOf('$POLY_ADDRESS').then(b => console.log((b / 1e6).toFixed(2))).catch(() => console.log('N/A'))\"" 2>/dev/null || echo "N/A") |

## 最近日志错误

\`\`\`
$(grep -E 'ERROR|panic|failed' "$LOG_DIR"/*.log 2>/dev/null | tail -20 || echo "无错误")
\`\`\`

## 测试日志文件

$(ls -la "$LOG_DIR/"*.log 2>/dev/null || echo "无日志文件")

## 建议操作

EOF

    # 根据检查结果添加建议
    if [ "$BINARY_EXISTS" != "yes" ]; then
        echo "1. **编译项目**: \`cd $PROJECT_PATH && cargo build --release\`" >> "$REPORT_FILE"
    fi
    
    if [ "$PRIVATE_KEY_SET" != "yes" ]; then
        echo "2. **设置私钥**: \`export POLYMARKET_PRIVATE_KEY='your_key'\`" >> "$REPORT_FILE"
    fi
    
    if [ "${MEMORY_AVAIL:-1000}" -lt 500 ]; then
        echo "3. **释放内存**: 关闭不必要的进程或增加 swap" >> "$REPORT_FILE"
    fi
    
    echo "" >> "$REPORT_FILE"
    echo "---" >> "$REPORT_FILE"
    echo "*报告由 vps-debug.sh 自动生成*" >> "$REPORT_FILE"
    
    print_success "诊断报告已保存：$REPORT_FILE"
}

# 主函数
main() {
    # 处理命令行参数 (可覆盖默认配置)
    if [ -n "$1" ]; then
        VPS_HOST="$1"
    fi
    if [ -n "$2" ]; then
        VPS_USER="$2"
    fi
    if [ -n "$3" ]; then
        VPS_KEY="$3"
    fi
    
    print_header "PolyMarket Knife VPS 调试工具"
    print_info "目标 VPS: $VPS_USER@$VPS_HOST"
    print_info "SSH 密钥：$VPS_KEY"
    print_info "项目路径：$PROJECT_PATH"
    print_info "日志目录：$LOG_DIR"
    print_info "测试钱包：$POLY_ADDRESS"
    echo ""
    print_info "默认配置 (可使用环境变量覆盖):"
    print_info "  VPS_HOST=$VPS_HOST"
    print_info "  VPS_USER=$VPS_USER"
    print_info "  VPS_KEY=$VPS_KEY"
    
    # 执行检查
    check_params
    setup_log_dir
    
    # 阶段 1-5: 快速检查
    test_ssh_connection || exit 1
    check_system_resources
    check_rust_environment
    check_project_status
    check_config_files
    
    # 阶段 6: 钱包余额检查
    check_wallet_balance
    
    # 询问是否继续
    echo ""
    read -p "是否继续执行主网测试？(y/N): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # 检查是否已部署
        BINARY_EXISTS=$(ssh $SSH_OPTS "$VPS_USER@$VPS_HOST" "test -f '$PROJECT_PATH/target/release/market-maker' && echo 'yes' || echo 'no'" 2>/dev/null)
        if [ "$BINARY_EXISTS" != "yes" ]; then
            print_warning "VPS 上未找到二进制文件"
            print_info "请先运行：./scripts/deploy-to-vps.sh"
            print_info "跳过主网测试"
        else
            test_mainnet_strategies
        fi
    fi
    
    # 生成报告
    generate_report
    
    print_header "调试完成"
    print_success "所有日志已保存到：$LOG_DIR"
    print_info "查看报告：cat $LOG_DIR/diagnostic_report_$TIMESTAMP.md"
    echo ""
    print_info "部署命令：./scripts/deploy-to-vps.sh"
    print_info "测试命令：./scripts/mainnet-test.sh"
}

# 运行主函数
main "$@"
