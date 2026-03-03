# VPS 调试快速参考

## 服务器信息

| 配置项 | 值 |
|--------|-----|
| **主机 IP** | `95.179.239.239` |
| **用户** | `root` |
| **SSH 密钥** | `~/works/agent-keys/agent` |
| **项目路径** | `/home/de/works/PolyMarket-knife` |

## 快速命令

### SSH 连接
```bash
# 直接连接
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent

# 使用脚本连接
./scripts/vps-quick-connect.sh

# 添加别名
echo "alias vps='ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent'" >> ~/.bashrc
```

### 运行调试脚本
```bash
# 使用默认配置
./scripts/vps-debug.sh

# 完整参数
./scripts/vps-debug.sh 95.179.239.239 root ~/works/agent-keys/agent

# 使用环境变量
export VPS_HOST="95.179.239.239"
export VPS_USER="root"
export VPS_KEY="$HOME/works/agent-keys/agent"
./scripts/vps-debug.sh
```

### 常用远程命令
```bash
# 定义变量
SSH="ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent"

# 查看系统状态
$SSH "free -h && df -h && uptime"

# 查看 Rust 环境
$SSH "rustc --version && cargo --version"

# 编译项目
$SSH "cd /home/de/works/PolyMarket-knife && cargo build --release"

# 运行程序
$SSH "cd /home/de/works/PolyMarket-knife && ./target/release/market-maker --config config/market-maker.toml"

# 查看日志
$SSH "tail -100f /home/de/works/PolyMarket-knife/app.log"

# 查看进程
$SSH "ps aux | grep market-maker"

# 重启程序
$SSH "cd /home/de/works/PolyMarket-knife && pkill -f market-maker; nohup ./target/release/market-maker --config config/market-maker.toml > app.log 2>&1 &"
```

### 文件传输
```bash
# SCP 上传文件
scp -o IdentitiesOnly=yes -i ~/works/agent-keys/agent \
    ./config/market-maker.toml \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/config/

# SCP 下载日志
scp -o IdentitiesOnly=yes -i ~/works/agent-keys/agent \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/*.log \
    ./logs/

# Rsync 同步项目
rsync -avz -e "ssh -i ~/works/agent-keys/agent -o IdentitiesOnly=yes" \
    ./ root@95.179.239.239:/home/de/works/PolyMarket-knife/
```

## 故障排查

### 连接问题
```bash
# 检查端口
nc -zv 95.179.239.239 22

# 检查网络延迟
ping -c 5 95.179.239.239

# 详细 SSH 调试
ssh -vvv root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent
```

### 编译问题
```bash
# 清理并重新编译
$SSH "cd /home/de/works/PolyMarket-knife && cargo clean && cargo build --release"

# 更新依赖
$SSH "cd /home/de/works/PolyMarket-knife && cargo update"

# 检查 Rust 版本
$SSH "rustc --version"

# 升级 Rust
$SSH "rustup update"
```

### 运行问题
```bash
# 设置环境变量
$SSH "export POLYMARKET_PRIVATE_KEY='your_key'"

# 带日志运行
$SSH "cd /home/de/works/PolyMarket-knife && RUST_LOG=debug ./target/release/market-maker --config config/market-maker.toml"

# 后台运行
$SSH "cd /home/de/works/PolyMarket-knife && nohup ./target/release/market-maker --config config/market-maker.toml > app.log 2>&1 &"

# 查看进程
$SSH "ps aux | grep market-maker"

# 查看实时日志
$SSH "tail -f /home/de/works/PolyMarket-knife/app.log"
```

## 监控命令

### 系统资源
```bash
# CPU 和内存
$SSH "top -bn1 | head -20"

# 磁盘空间
$SSH "df -h"

# 网络连接
$SSH "netstat -tulpn | grep LISTEN"

# 系统负载
$SSH "uptime"
```

### 应用监控
```bash
# 查看运行进程
$SSH "ps aux | grep -E 'market-maker|arbitrage|follow-trade'"

# 查看打开的文件
$SSH "lsof -p \$(pgrep market-maker)"

# 查看网络连接
$SSH "ss -tulpn | grep market-maker"
```

## 自动化脚本

### 一键部署
```bash
#!/bin/bash
SSH="ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent"

echo "同步代码..."
rsync -avz -e "ssh -i ~/works/agent-keys/agent -o IdentitiesOnly=yes" \
    ./ root@95.179.239.239:/home/de/works/PolyMarket-knife/

echo "编译项目..."
$SSH "cd /home/de/works/PolyMarket-knife && cargo build --release"

echo "重启服务..."
$SSH "cd /home/de/works/PolyMarket-knife && pkill -f market-maker; nohup ./target/release/market-maker --config config/market-maker.toml > app.log 2>&1 &"

echo "完成!"
```

### 健康检查
```bash
#!/bin/bash
SSH="ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent"

echo "=== 系统状态 ==="
$SSH "uptime && free -h && df -h /home"

echo "=== 进程状态 ==="
$SSH "ps aux | grep market-maker | grep -v grep"

echo "=== 最近日志 ==="
$SSH "tail -20 /home/de/works/PolyMarket-knife/app.log"
```

---

**最后更新**: 2026-03-03
**服务器**: 95.179.239.239
