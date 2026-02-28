# 常见问题解答 (FAQ)

## 安装部署

### Q: Rust 版本要求？

**A**: 需要 Rust 1.75 或更高版本。

```bash
rustc --version
rustup update  # 更新到最新版本
```

### Q: 编译失败怎么办？

**A**: 常见原因：
1. Rust 版本过低 - 运行 `rustup update`
2. 依赖冲突 - 运行 `cargo clean && cargo build`
3. 系统依赖缺失 - 安装 `pkg-config` 和 `libssl-dev`

### Q: 如何部署到服务器？

**A**: 
```bash
# 1. 编译
cargo build --release

# 2. 复制二进制和配置
scp target/release/market-maker user@server:~/
scp config/*.toml user@server:~/config/

# 3. 运行
./market-maker --config config/market-maker.toml
```

## 配置问题

### Q: 私钥如何安全存储？

**A**: 三种方式：
1. **环境变量** (推荐)
   ```bash
   export POLYMARKET_PRIVATE_KEY="your_key"
   ```

2. **配置文件** (需要设置权限)
   ```bash
   chmod 600 config/market-maker.toml
   ```

3. **密钥管理服务** (生产环境)
   - AWS Secrets Manager
   - HashiCorp Vault

### Q: 如何配置多个市场？

**A**: 
```toml
[strategy]
market_ids = [
    "0x123...",  # 市场 1
    "0x456...",  # 市场 2
    "0x789...",  # 市场 3
]
```

### Q: 如何调整风险参数？

**A**: 
```toml
[risk]
max_position_usd = 10000      # 最大持仓
max_loss_per_day = 500        # 日最大亏损
stop_loss_pct = 5.0           # 止损百分比
```

## 策略使用

### Q: 哪个策略最适合新手？

**A**: 推荐顺序：
1. **跟单策略** - 最简单，复制聪明钱
2. **套利策略** - 低风险，稳定收益
3. **做市策略** - 中等风险，稳定收益
4. **波动狩猎** - 高风险，高收益

### Q: 需要多少资金起步？

**A**: 
- 跟单：$1,000+
- 套利：$5,000+
- 做市：$10,000+
- 波动狩猎：$50,000+

### Q: 预期收益率是多少？

**A**: 
- 跟单：50%-150%/年
- 套利：20%-50%/年
- 做市：30%-80%/年
- 波动狩猎：100%-500%/年 (高风险)

## 交易问题

### Q: 订单不成交怎么办？

**A**: 
1. 检查价格是否有竞争力
2. 增加订单大小
3. 调整价差参数
4. 检查市场流动性

### Q: 如何查看订单状态？

**A**: 
```bash
# 查看日志
tail -f market-maker.log | grep "Order"

# 或使用 API
curl -H "Authorization: Bearer $API_KEY" \
  https://clob.polymarket.com/orders
```

### Q: 如何停止交易？

**A**: 
```bash
# 方法 1: Ctrl+C
./market-maker  # 按 Ctrl+C

# 方法 2: 发送信号
kill -SIGINT <pid>

# 方法 3: API 调用
curl -X POST http://localhost:8080/stop
```

## 监控告警

### Q: 如何设置告警通知？

**A**: 
```toml
[alerts]
email = "your@email.com"
slack_webhook = "https://hooks.slack.com/..."
telegram_bot = "your_bot_token"
```

### Q: 日亏损阈值如何设置？

**A**: 建议设置为总资金的 5%-10%。

```toml
[risk]
max_loss_per_day = 500  # 如果总资金$5000，设置为 10%
```

### Q: 如何查看监控指标？

**A**: 
```bash
# Prometheus 指标
curl http://localhost:9090/metrics

# Grafana 仪表板
open http://localhost:3000
```

## 性能优化

### Q: 如何降低延迟？

**A**: 
1. 使用低延迟 VPS (推荐 AWS us-east-1)
2. 启用连接池
3. 使用 WebSocket 代替 REST
4. CPU 绑定优化

### Q: 内存使用过高怎么办？

**A**: 
1. 减少缓存大小
2. 调整对象池容量
3. 检查内存泄漏
4. 使用 `valgrind` 分析

### Q: CPU 使用率过高？

**A**: 
1. 减少刷新频率
2. 优化循环逻辑
3. 使用 CPU 绑定
4. 分析热点函数

## 故障排查

### Q: 程序崩溃怎么办？

**A**: 
```bash
# 1. 查看日志
tail -f market-maker.log

# 2. 生成核心转储
ulimit -c unlimited
# 程序崩溃后分析
gdb ./market-maker core

# 3. 报告问题
# 在 GitHub 提交 issue，附带日志
```

### Q: 连接断开怎么办？

**A**: 
1. 检查网络连接
2. 检查防火墙设置
3. 增加重试次数
4. 启用自动重连

### Q: 数据不一致怎么办？

**A**: 
1. 重启程序同步数据
2. 检查 API 返回
3. 清除本地缓存
4. 联系技术支持

## 安全相关

### Q: 如何保证资金安全？

**A**: 
1. 使用硬件钱包
2. 设置取款白名单
3. 启用双因素认证
4. 定期更换密钥

### Q: 私钥泄露怎么办？

**A**: 
1. **立即**转移资金到新地址
2. 撤销 API 密钥
3. 更换私钥
4. 检查交易记录

### Q: 如何防止未授权访问？

**A**: 
1. 设置防火墙规则
2. 使用 VPN
3. 启用 IP 白名单
4. 定期审计日志

## 其他问题

### Q: 支持哪些交易所？

**A**: 目前仅支持 Polymarket。

### Q: 可以自定义策略吗？

**A**: 可以，参考 `docs/ARCHITECTURE.md` 了解架构。

### Q: 有社区支持吗？

**A**: 
- GitHub Issues: 提问和反馈
- Discord: 实时交流
- Email: developer@polymarket-knife.dev

### Q: 如何贡献代码？

**A**: 
1. Fork 仓库
2. 创建分支
3. 提交代码
4. 创建 Pull Request

参考 `CONTRIBUTING.md` 了解详情。

