# 部署指南

## 系统要求

- **OS**: Ubuntu 22.04 LTS 或更高
- **CPU**: 4 核以上 (推荐 8 核)
- **内存**: 8GB 以上
- **网络**: 低延迟连接 (推荐 AWS us-east-1)
- **Rust**: 1.75+

## 快速部署

### 1. 安装依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装系统依赖
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

### 2. 编译

```bash
cd /home/de/works/PolyMarket-knife

# 编译所有程序
cargo build --release

# 或编译单个程序
cargo build --release -p market-maker
```

### 3. 配置

```bash
# 复制配置模板
cp config/market-maker.toml.example config/market-maker.toml

# 编辑配置
vim config/market-maker.toml
```

### 4. 运行

```bash
# 前台运行
./target/release/market-maker --config config/market-maker.toml

# 后台运行 (使用 systemd)
sudo systemctl start market-maker

# 或使用 screen/tmux
screen -S mm
./target/release/market-maker --config config/market-maker.toml
```

## 生产环境优化

### CPU 优化

```bash
# 1. 识别 CPU 核心
lscpu

# 2. 隔离 CPU 核心 (编辑 /etc/default/grub)
GRUB_CMDLINE_LINUX="isolcpus=2,3"
sudo update-grub
sudo reboot

# 3. 绑定进程到特定核心
taskset -c 2 ./target/release/market-maker
```

### 网络优化

```bash
# 1. 禁用中断合并
ethtool -K eth0 gro off
ethtool -K eth0 gso off
ethtool -K eth0 tso off

# 2. 增加文件描述符限制
ulimit -n 65536

# 3. 调整 TCP 参数
sudo sysctl -w net.ipv4.tcp_fastopen=3
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728
```

### 内存优化

```bash
# 1. 锁定内存 (防止 swap)
ulimit -l unlimited

# 2. 在 systemd 服务中配置
# /etc/systemd/system/market-maker.service
[Service]
LimitMEMLOCK=infinity
```

## Systemd 服务配置

### market-maker.service

```ini
[Unit]
Description=PolyMarket Market Maker
After=network.target

[Service]
Type=simple
User=de
WorkingDirectory=/home/de/works/PolyMarket-knife
ExecStart=/home/de/works/PolyMarket-knife/target/release/market-maker --config config/market-maker.toml
Restart=always
RestartSec=5
LimitNOFILE=65536
LimitMEMLOCK=infinity

# CPU 绑定
CPUAffinity=2

# 环境变量
Environment="RUST_LOG=info"
Environment="PRIVATE_KEY=your_key_here"

[Install]
WantedBy=multi-user.target
```

### 启用服务

```bash
sudo cp market-maker.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable market-maker
sudo systemctl start market-maker

# 查看状态
sudo systemctl status market-maker

# 查看日志
sudo journalctl -u market-maker -f
```

## 监控配置

### Prometheus 配置

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'market-maker'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'

  - job_name: 'volatility-hunter'
    static_configs:
      - targets: ['localhost:9091']
    metrics_path: '/metrics'
```

### Grafana 仪表板

导入仪表板 ID:
- Market Maker: `TODO`
- Volatility Hunter: `TODO`

## 告警配置

```yaml
# alertmanager.yml
groups:
  - name: polymarket
    rules:
      - alert: HighLatency
        expr: market_maker_latency_ms > 100
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Market maker latency too high"

      - alert: HighLoss
        expr: market_maker_daily_pnl < -500
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Market maker daily loss exceeded"
```

## 备份与恢复

### 配置备份

```bash
# 备份配置
tar -czf config-backup-$(date +%Y%m%d).tar.gz config/

# 上传到 S3
aws s3 cp config-backup.tar.gz s3://your-bucket/backups/
```

### 密钥管理

```bash
# 使用 AWS Secrets Manager
aws secretsmanager create-secret \
  --name polymarket/private-key \
  --secret-string "your-private-key"

# 运行时获取
export PRIVATE_KEY=$(aws secretsmanager get-secret-value \
  --secret-id polymarket/private-key \
  --query SecretString --output text)
```

## 故障排除

### 常见问题

1. **连接超时**
   ```bash
   # 检查网络
   ping polygon-rpc.com
   
   # 检查防火墙
   sudo ufw status
   ```

2. **内存不足**
   ```bash
   # 检查内存使用
   free -h
   
   # 检查进程
   ps aux | grep market-maker
   ```

3. **延迟过高**
   ```bash
   # 检查 CPU 使用
   top -H -p $(pgrep market-maker)
   
   # 检查网络延迟
   mtr polygon-rpc.com
   ```

### 日志分析

```bash
# 搜索错误
sudo journalctl -u market-maker | grep ERROR

# 分析性能
sudo journalctl -u market-maker | grep "latency"
```
