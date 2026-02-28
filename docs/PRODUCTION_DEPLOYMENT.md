# 生产部署指南

## ⚠️ 重要警告

**当前状态**: 框架就绪，业务逻辑待实现

**可以部署**:
- ✅ 框架代码
- ✅ 监控系统
- ✅ 风控系统
- ✅ 密钥管理

**不能部署**:
- ❌ 真实交易 (下单逻辑未完成)
- ❌ 实盘资金

---

## 部署清单

### 1. 环境准备

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装系统依赖
sudo apt install build-essential pkg-config libssl-dev

# 验证版本
rustc --version  # >= 1.75
```

### 2. 密钥配置

```bash
# 生成密钥
./scripts/setup-keys.sh generate

# 存储到 AWS Secrets Manager
aws secretsmanager create-secret \
  --name polymarket/private-key \
  --secret-string "0x..."

# 验证密钥
./scripts/setup-keys.sh validate
```

### 3. 编译

```bash
# 编译 release 版本
cargo build --release

# 验证二进制
./target/release/market-maker --version
```

### 4. 配置

```bash
# 复制配置模板
cp config/market-maker.toml.example config/market-maker.toml

# 编辑配置
vim config/market-maker.toml

# 设置环境变量
export POLYMARKET_PRIVATE_KEY=$(aws secretsmanager get-secret-value \
  --secret-id polymarket/private-key \
  --query SecretString \
  --output text)
```

### 5. 部署

```bash
# 使用 systemd
sudo cp market-maker.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable market-maker
sudo systemctl start market-maker

# 验证状态
sudo systemctl status market-maker
```

### 6. 监控

```bash
# 访问 Prometheus 指标
curl http://localhost:9090/metrics

# 预期输出:
# market_maker_orders_placed 0
# market_maker_orders_filled 0
# market_maker_daily_pnl 0
```

---

## 系统架构

```
┌────────────────────────────────────────┐
│          Market Maker                   │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌────────┐ │
│  │ Config  │  │Executor │  │ Risk   │ │
│  └─────────┘  └─────────┘  └────────┘ │
│  ┌─────────┐  ┌─────────┐  ┌────────┐ │
│  │ Quoter  │  │ Metrics │  │ Logger │ │
│  └─────────┘  └─────────┘  └────────┘ │
└────────────────────────────────────────┘
           │
           │ HTTP:9090
           ▼
┌────────────────────────────────────────┐
│          Prometheus                     │
│          (指标收集)                      │
└────────────────────────────────────────┘
           │
           ▼
┌────────────────────────────────────────┐
│           Grafana                       │
│          (可视化)                        │
└────────────────────────────────────────┘
```

---

## 监控指标

### Prometheus 指标

| 指标 | 类型 | 说明 |
|------|------|------|
| market_maker_orders_placed | Counter | 下单总数 |
| market_maker_orders_filled | Counter | 成交总数 |
| market_maker_orders_cancelled | Counter | 取消总数 |
| market_maker_orders_failed | Counter | 失败总数 |
| market_maker_daily_pnl | Gauge | 日 PnL (分) |
| market_maker_daily_volume | Counter | 日成交量 (分) |
| market_maker_last_update | Gauge | 最后更新时间 |

### Grafana 仪表板

导入仪表板 ID: `TODO`

**关键面板**:
- PnL 趋势
- 订单统计
- 延迟监控
- 错误率

---

## 告警规则

```yaml
# alertmanager.yml
groups:
  - name: market-maker
    rules:
      - alert: HighErrorRate
        expr: rate(market_maker_orders_failed[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High order failure rate"
          
      - alert: HighLoss
        expr: market_maker_daily_pnl < -50000  # $500
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Daily loss exceeded threshold"
          
      - alert: ServiceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Market maker service is down"
```

---

## 故障排查

### 服务无法启动

```bash
# 查看日志
journalctl -u market-maker -f

# 检查配置
market-maker --config config/market-maker.toml

# 验证密钥
./scripts/setup-keys.sh validate
```

### 指标无法访问

```bash
# 检查端口
netstat -tlnp | grep 9090

# 检查防火墙
sudo ufw status

# 测试连接
curl http://localhost:9090/metrics
```

### 订单失败

```bash
# 查看日志
journalctl -u market-maker | grep "Failed"

# 检查 API 连接
curl -I https://clob.polymarket.com

# 验证余额
# (需要实现余额查询)
```

---

## 安全最佳实践

### 1. 密钥安全

- ✅ 使用 Secrets Manager
- ✅ 定期轮换密钥
- ✅ 最小权限原则
- ✅ 访问审计

### 2. 网络安全

- ✅ 防火墙限制
- ✅ TLS 加密
- ✅ VPC 隔离
- ✅ 私有子网

### 3. 系统安全

- ✅ 定期更新
- ✅ 最小化安装
- ✅ 入侵检测
- ✅ 日志审计

### 4. 应用安全

- ✅ 输入验证
- ✅ 错误处理
- ✅ 速率限制
- ✅ 资源限制

---

## 应急计划

### 密钥泄露

1. **立即撤销** - Polymarket 平台
2. **生成新密钥** - ./scripts/setup-keys.sh generate
3. **更新配置** - 所有环境
4. **审查日志** - 检查异常
5. **通知相关方** - 安全团队

### 服务故障

1. **检查状态** - systemctl status
2. **查看日志** - journalctl -f
3. **重启服务** - systemctl restart
4. **回滚版本** - 如有必要
5. **通知用户** - 如影响交易

### 资金风险

1. **停止交易** - systemctl stop
2. **评估损失** - 检查持仓和 PnL
3. **调整风控** - 降低限额
4. **报告事件** - 管理层
5. **事后分析** - 改进措施

---

## 检查清单

### 部署前

- [ ] 密钥已安全存储
- [ ] 配置已验证
- [ ] 测试通过
- [ ] 监控配置
- [ ] 告警配置
- [ ] 备份策略
- [ ] 应急计划

### 部署后

- [ ] 服务状态正常
- [ ] 指标正常上报
- [ ] 日志正常记录
- [ ] 告警测试通过
- [ ] 性能符合预期
- [ ] 安全扫描通过

### 运行中

- [ ] 每日检查日志
- [ ] 每周审查指标
- [ ] 每月轮换密钥
- [ ] 每季度安全审计
- [ ] 每半年应急演练

---

**生产部署是严肃的事情，请谨慎操作！**

