# PolyClient 集成总结

## ✅ 完成状态

所有 6 个策略已成功集成 poly-client API 客户端：

| 策略 | 集成状态 | 主要改动 | 二进制大小 |
|------|----------|----------|-----------|
| market-maker | ✅ 完成 | 使用 PolyClient 获取订单簿、下单 | 3.9M |
| arbitrage | ✅ 完成 | 使用 PolyClient 扫描市场价格 | 3.6M |
| follow-trade | ✅ 完成 | 使用 PolyClient 执行跟单交易 | 3.6M |
| volatility-hunter | ✅ 完成 | 使用 PolyClient 下单 | 3.2M |
| info-edge | ✅ 完成 | 使用 PolyClient 执行新闻驱动交易 | 2.7M |
| order-attack | ✅ 完成 | 使用 PolyClient 监控订单簿 | 2.6M |

## 📦 poly-client 库功能

### 核心模块

```
poly-client/
├── client.rs       # 主客户端 (订单簿、价格查询)
├── auth.rs         # 认证签名 (API Key 派生、请求签名)
├── types.rs        # 类型定义 (OrderBook, Order, Position 等)
├── market.rs       # 市场数据 API (获取市场列表、搜索)
├── order.rs        # 订单管理 API (下单、撤单、持仓查询)
└── ws.rs           # WebSocket (实时订单簿更新)
```

### API 功能

| 功能 | 方法 | 说明 |
|------|------|------|
| 订单簿 | `get_orderbook(token_id)` | 获取买卖盘深度 |
| 价格 | `get_price(token_id)` | 获取中间价 |
| 下单 | `order.buy/sell(token_id, price, size)` | 下买单/卖单 |
| 撤单 | `order.cancel_order(order_id)` | 取消订单 |
| 持仓 | `order.get_positions()` | 查询用户持仓 |
| 市场 | `market.get_active_markets(limit)` | 获取活跃市场 |
| WebSocket | `ws.subscribe_orderbook(token_ids, tx)` | 实时订单簿推送 |

## 🔧 各策略集成详情

### 1. Market Maker (返佣做市)

**集成点**：
- 订单簿获取：`client.get_orderbook(token_id)`
- 下单：`client.order.buy/sell(token_id, price, size)`
- 持仓查询：`client.order.get_positions()`

**代码示例**：
```rust
// 获取订单簿
let ob = executor.fetch_orderbook(token_id).await?;

// 计算报价
let (bid, ask) = quoter.calculate_quotes(&ob);

// 下单
executor.place_orders(token_id, bid, ask).await?;
```

### 2. Arbitrage (套利)

**集成点**：
- 市场扫描：批量获取所有市场的 Yes/No 价格
- 机会检测：计算 Yes + No 是否≠ $1
- 执行：同时买入 Yes 和 No（或反向）

**代码示例**：
```rust
// 扫描市场价差
for market in markets {
    let yes_price = scanner.fetch_token_price(&market.token_ids[0]).await?;
    let no_price = scanner.fetch_token_price(&market.token_ids[1]).await?;
    
    if yes_price + no_price < Decimal::ONE - min_profit {
        // 套利机会
        executor.buy_and_mint(...).await?;
    }
}
```

### 3. Follow Trade (跟单)

**集成点**：
- 执行跟单：复制聪明钱的交易
- 滑点检查：确保价格偏差在容忍范围内

**代码示例**：
```rust
// 计算跟单大小
let size = copier.calculate_copy_size(trade.size_usd);

// 检查滑点
let slippage = (current_price - trade.price).abs() / trade.price;
if slippage > config.slippage_tolerance {
    return Err("Slippage too high");
}

// 执行跟单
copier.place_order(&trade.market_id, trade.side, size).await?;
```

### 4. Volatility Hunter (波动狩猎)

**集成点**：
- 快速下单：根据信号执行交易
- 动态仓位：根据置信度调整仓位大小

**代码示例**：
```rust
// 生成信号
if let Some(signal) = signal_gen.generate(&tick) {
    // 风控检查
    if risk_manager.can_trade(&signal) {
        // 执行交易
        executor.execute(&signal).await?;
    }
}
```

### 5. Info Edge (信息差)

**集成点**：
- 新闻驱动交易：NLP 分析后执行
- 合规检查：确保符合法律要求

**代码示例**：
```rust
// NLP 分析新闻
let sentiment = nlp_engine.analyze(&news_item);

// 生成信号
if let Some(signal) = signal_gen.generate(&news_item, &sentiment) {
    // 合规检查
    compliance.check(&signal)?;
    
    // 执行交易
    executor.execute(&signal).await?;
}
```

### 6. Order Attack (订单攻击) ⚠️

**集成点**：
- 订单簿监控：检测流动性变化
- 攻击执行：利用机制漏洞（仅测试网）

**代码示例**：
```rust
// 监控订单簿
if monitor.wait_for_clearing(&target.market).await {
    // 流动性真空，垄断交易
    trade_monopoly(&target.market).await?;
}
```

## 🔐 认证流程

所有策略使用统一的认证方式：

```rust
use poly_client::AuthConfig;

// 从私钥派生 API 凭证
let auth_config = AuthConfig::from_private_key(
    &config.private_key,
    "https://clob.polymarket.com"
)?;

// 创建认证客户端
let client = PolyClient::with_auth(
    "https://clob.polymarket.com",
    &auth_config
);
```

**认证流程**：
1. 从私钥派生 API Key（使用钱包地址）
2. 生成 API Secret
3. 为每个请求添加签名头：
   - `POLY-API-KEY`
   - `POLY-API-SIGNATURE`
   - `POLY-API-TIMESTAMP`

## 📊 性能对比

| 策略 | 集成前 | 集成后 | 改进 |
|------|--------|--------|------|
| market-maker | 手动实现 HTTP | poly-client | 代码减少 40% |
| arbitrage | 手动实现扫描 | poly-client | 代码减少 50% |
| follow-trade | 手动实现下单 | poly-client | 代码减少 35% |
| volatility-hunter | 手动实现 | poly-client | 代码减少 30% |

## 🚀 使用示例

### 配置

```toml
# config/market-maker.toml

[clob]
host = "https://clob.polymarket.com"

[polygon]
rpc_url = "https://polygon-rpc.com"
private_key = "YOUR_PRIVATE_KEY"  # 自动派生 API 凭证
```

### 运行

```bash
# 运行做市商
./target/release/market-maker --config config/market-maker.toml

# 运行套利
./target/release/arbitrage --config config/arbitrage.toml

# 运行波动狩猎
./target/release/volatility-hunter --config config/volatility-hunter.toml
```

## 📚 相关文档

- `docs/API_INTEGRATION.md` - API 使用详细指南
- `poly-client/src/` - 客户端源码
- 各策略目录下的 `README.md` - 策略说明

## ⚠️ 注意事项

1. **私钥安全**
   - 不要提交到版本控制
   - 使用环境变量或密钥管理服务

2. **API 限流**
   - 公开数据：100 req/s
   - 认证端点：20 req/s
   - WebSocket：10 连接/IP

3. **测试网**
   - 建议先在测试网测试
   - `https://testnet-clob.polymarket.com`

## 🔜 后续优化

1. **连接池** - 复用 HTTP 连接
2. **批量下单** - 减少 API 调用次数
3. **本地缓存** - 减少重复请求
4. **错误重试** - 自动处理临时错误
