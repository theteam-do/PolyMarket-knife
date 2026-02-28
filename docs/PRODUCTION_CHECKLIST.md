# 生产环境部署检查清单

## 📋 部署前检查

### 安全配置

- [ ] 私钥安全存储
- [ ] API 密钥轮换计划
- [ ] 防火墙规则配置
- [ ] 访问控制列表 (ACL)
- [ ] 审计日志启用

### 系统配置

- [ ] Rust 版本 >= 1.75
- [ ] 系统依赖安装
- [ ] 文件描述符限制 (65536+)
- [ ] 内存锁定启用
- [ ] CPU 隔离配置

### 网络配置

- [ ] 低延迟 VPS (AWS us-east-1 推荐)
- [ ] 网络优化 (ethtool)
- [ ] DNS 配置
- [ ] NTP 时间同步
- [ ] 备用网络线路

### 监控告警

- [ ] Prometheus 配置
- [ ] Grafana 仪表板
- [ ] 告警规则配置
- [ ] 通知渠道 (邮件/Slack/Telegram)
- [ ] 健康检查端点

---

## 🔐 密钥管理方案

### 方案 A: 环境变量 (推荐)

```bash
# .env.production
POLYMARKET_PRIVATE_KEY="0x..."
CLOB_API_KEY="..."
CLOB_API_SECRET="..."

# 加载环境变量
source .env.production
export POLYMARKET_PRIVATE_KEY
```

### 方案 B: AWS Secrets Manager

```rust
use aws_sdk_secretsmanager::Client;

async fn get_secret(secret_name: &str) -> Result<String> {
    let client = Client::new(&config);
    let resp = client.get_secret_value().secret_id(secret_name).send().await?;
    Ok(resp.secret_string.unwrap())
}
```

### 方案 C: HashiCorp Vault

```bash
# 启动 Vault
vault server -dev

# 存储密钥
vault kv put secret/polymarket private_key="0x..."

# 读取密钥
vault kv get secret/polymarket
```

---

## 📝 生产配置模板

### config/production.toml

```toml
[polygon]
rpc_url = "https://polygon-rpc.com"
# 私钥从环境变量读取

[clob]
host = "https://clob.polymarket.com"

[strategy]
market_ids = [
    "0x...",  # 实际市场 ID
]
spread_bps = 100
order_size_usd = 1000
refresh_interval_ms = 100
skew_inventory = true

[risk]
max_position_usd = 10000
max_loss_per_day = 500
stop_loss_pct = 5.0
max_orders = 10
max_order_size_usd = 5000

[monitoring]
prometheus_port = 9090
health_check_port = 8080
log_level = "info"
```

### systemd 服务配置

```ini
# /etc/systemd/system/market-maker.service
[Unit]
Description=PolyMarket Market Maker
After=network.target

[Service]
Type=simple
User=polymarket
WorkingDirectory=/opt/polymarket-knife
ExecStart=/opt/polymarket-knife/target/release/market-maker --config config/production.toml
Restart=always
RestartSec=5
LimitNOFILE=65536
LimitMEMLOCK=infinity

# 环境变量
Environment="POLYMARKET_PRIVATE_KEY=${POLYMARKET_PRIVATE_KEY}"
Environment="RUST_LOG=info"

# 安全设置
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/polymarket

[Install]
WantedBy=multi-user.target
```

---

## 🚀 部署脚本

### deploy.sh

```bash
#!/bin/bash
set -e

echo "Deploying PolyMarket Knife..."

# 1. 拉取最新代码
git pull origin main

# 2. 编译 release 版本
cargo build --release

# 3. 停止旧服务
sudo systemctl stop market-maker

# 4. 备份旧版本
sudo cp target/release/market-maker target/release/market-maker.bak

# 5. 部署新版本
sudo cp target/release/market-maker /opt/polymarket-knife/

# 6. 启动新服务
sudo systemctl start market-maker

# 7. 检查服务状态
sudo systemctl status market-maker

# 8. 运行健康检查
curl -f http://localhost:8080/health || exit 1

echo "Deployment complete!"
```

### rollback.sh

```bash
#!/bin/bash
set -e

echo "Rolling back..."

# 恢复旧版本
sudo cp target/release/market-maker.bak /opt/polymarket-knife/market-maker

# 重启服务
sudo systemctl restart market-maker

echo "Rollback complete!"
```

---

## 📊 监控告警集成

### Prometheus 配置

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'market-maker'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
    
  - job_name: 'arbitrage'
    static_configs:
      - targets: ['localhost:9091']
    scrape_interval: 15s
```

### Grafana 仪表板

导入仪表板 ID: `TODO`

**关键指标**:
- PnL 趋势
- 订单延迟
- API 错误率
- 持仓变化
- 告警数量

### 告警规则

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
          
      - alert: ServiceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service is down"
```

---

## 🔒 安全检查清单

### 系统安全

- [ ] 防火墙配置 (仅允许必要端口)
- [ ] SSH 密钥认证
- [ ] 禁用 root 登录
- [ ] 定期系统更新
- [ ] 入侵检测系统

### 应用安全

- [ ] 输入验证
- [ ] 错误处理 (不泄露敏感信息)
- [ ] 日志脱敏
- [ ] 速率限制
- [ ] CSRF 保护

### 数据安全

- [ ] 加密存储
- [ ] 加密传输 (TLS)
- [ ] 密钥轮换
- [ ] 访问审计
- [ ] 备份策略

---

## 📈 性能基准

### 目标性能指标

| 指标 | 目标值 | 告警阈值 |
|------|--------|----------|
| API 延迟 | <50ms | >100ms |
| 下单延迟 | <100ms | >200ms |
| WebSocket 延迟 | <20ms | >50ms |
| 内存使用 | <50MB | >100MB |
| CPU 使用 | <20% | >50% |

### 压力测试

```bash
# 运行压力测试
./scripts/stress-test.sh

# 目标：1000 并发请求
# 成功率：>99.9%
# P99 延迟：<200ms
```

---

## 🆘 应急响应

### 故障处理流程

1. **检测**: 监控告警触发
2. **评估**: 确定影响范围
3. **响应**: 启动应急预案
4. **恢复**: 修复问题/回滚
5. **复盘**: 事后分析

### 联系人列表

| 角色 | 姓名 | 联系方式 |
|------|------|----------|
| On-call | TODO | TODO |
| 技术负责人 | TODO | TODO |
| 安全负责人 | TODO | TODO |

### 回滚计划

```bash
# 快速回滚
./scripts/rollback.sh

# 验证回滚
curl http://localhost:8080/health
```

---

## ✅ 部署验证

### 健康检查

```bash
# 检查服务状态
curl http://localhost:8080/health

# 预期响应
{"status": "healthy", "version": "0.1.0"}
```

### 功能验证

- [ ] 获取订单簿成功
- [ ] 下单成功
- [ ] 撤单成功
- [ ] 监控指标正常
- [ ] 告警系统正常

### 性能验证

- [ ] API 延迟 <50ms
- [ ] 下单延迟 <100ms
- [ ] 内存使用 <50MB
- [ ] CPU 使用 <20%

---

**部署完成后，持续监控 24 小时，确保系统稳定运行！**

