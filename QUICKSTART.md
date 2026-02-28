# 快速开始指南

## 5 分钟上手

### 1. 检查环境

```bash
# 需要 Rust 1.75+
rustc --version

# 如果没有安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 编译

```bash
cd /home/de/works/PolyMarket-knife

# 编译所有程序 (约 5-10 分钟)
cargo build --release

# 或只编译一个程序
cargo build --release -p market-maker
```

### 3. 配置

```bash
# 复制配置模板
cp config/market-maker.toml.example config/market-maker.toml

# 编辑配置 (至少设置私钥)
vim config/market-maker.toml
```

### 4. 运行

```bash
# 运行做市商
./target/release/market-maker --config config/market-maker.toml

# 运行波动狩猎
./target/release/volatility-hunter --config config/volatility-hunter.toml

# 运行跟单
./target/release/follow-trade --config config/follow-trade.toml
```

## 选择你的第一个策略

### 🟢 新手推荐：跟单

最简单，适合学习：

```bash
# 1. 编辑配置
vim config/follow-trade.toml

# 2. 添加聪明钱地址
smart_addresses = [
    "0x...",  # 替换为实际地址
]

# 3. 运行
./target/release/follow-trade --config config/follow-trade.toml
```

### 🟡 进阶推荐：返佣做市

稳定收益：

```bash
# 1. 编辑配置
vim config/market-maker.toml

# 2. 设置做市参数
spread_bps = 100          # 1% 价差
order_size_usd = 1000     # 单笔大小

# 3. 运行
./target/release/market-maker --config config/market-maker.toml
```

### 🔴 高级推荐：波动狩猎

高收益高风险：

```bash
# 1. 编辑配置
vim config/volatility-hunter.toml

# 2. 配置币安 API
api_key = "..."
api_secret = "..."

# 3. 运行
./target/release/volatility-hunter --config config/volatility-hunter.toml
```

## 验证运行

### 检查日志

```bash
# 日志应该显示类似:
# INFO market_maker: Market Maker starting...
# INFO market_maker: Tick completed
```

### 检查指标 (如果配置了 Prometheus)

```bash
curl localhost:9090/metrics
```

## 常见问题

### 编译错误

```bash
# 更新 Rust
rustup update

# 清除缓存重新编译
cargo clean && cargo build --release
```

### 连接错误

```bash
# 检查网络
ping polygon-rpc.com

# 更换 RPC 节点
# 在配置文件中修改 rpc_url
```

### 交易失败

```bash
# 检查余额
# 确保账户有足够 POL 支付 Gas

# 检查配置
# 确认 private_key 正确
```

## 下一步

1. ✅ 成功运行一个策略
2. 📖 阅读对应策略的详细文档
3. 🧪 小额测试 1-2 周
4. 📊 监控收益和风险
5. 🚀 逐步增加资金或添加新策略

## 获取帮助

- 📄 查看 `docs/` 目录的详细文档
- 🐛 遇到问题检查日志
- 💬 加入社区讨论

祝你交易顺利！🎯
