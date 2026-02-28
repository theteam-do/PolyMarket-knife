# Polymarket API 对接文档

## 📦 poly-client 库

完整的 Polymarket CLOB API 客户端实现，包含：

### 模块结构

```
poly-client/
├── src/
│   ├── lib.rs          # 库入口
│   ├── client.rs       # 主客户端
│   ├── auth.rs         # 认证/签名
│   ├── types.rs        # 类型定义
│   ├── market.rs       # 市场数据 API
│   ├── order.rs        # 订单管理 API
│   └── ws.rs           # WebSocket 实时数据
```

### 核心功能

#### 1. 认证 (auth.rs)

```rust
use poly_client::AuthConfig;

// 从私钥派生 API 凭证
let auth_config = AuthConfig::from_private_key(
    "YOUR_PRIVATE_KEY",
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

#### 2. 市场数据 (market.rs)

```rust
// 获取活跃市场
let markets = client.market.get_active_markets(100).await?;

// 获取特定市场
let market = client.market.get_market("CONDITION_ID").await?;

// 按标签搜索
let markets = client.market.get_markets_by_tag("politics", 50).await?;

// 搜索市场
let markets = client.market.search_markets("election", 20).await?;
```

**API 端点**：
- `GET /markets` - 市场列表
- `GET /markets/{id}` - 单个市场
- `GET /search` - 搜索

#### 3. 订单簿数据 (client.rs)

```rust
// 获取订单簿
let orderbook = client.get_orderbook("TOKEN_ID").await?;

// 获取中间价
let price = client.get_price("TOKEN_ID").await?;

// 批量获取
let orderbooks = client.get_orderbooks(&["TOKEN1", "TOKEN2"]).await?;
```

**订单簿结构**：
```rust
pub struct OrderBook {
    pub token_id: String,
    pub bids: Vec<OrderBookLevel>,  // 买单
    pub asks: Vec<OrderBookLevel>,  // 卖单
    pub timestamp: u64,
}

pub struct OrderBookLevel {
    pub price: Decimal,
    pub size: Decimal,
}
```

#### 4. 订单管理 (order.rs)

```rust
// 下买单
let resp = client.order.buy(
    "TOKEN_ID",
    Decimal::from_str("0.50")?,
    Decimal::from_str("100")?
).await?;

// 下卖单
let resp = client.order.sell(
    "TOKEN_ID",
    Decimal::from_str("0.55")?,
    Decimal::from_str("100")?
).await?;

// 取消订单
client.order.cancel_order("ORDER_ID").await?;

// 获取订单
let orders = client.order.get_orders(Some("MARKET_ID")).await?;
```

**订单类型**：
- `Gtc` - Good Till Cancel（默认）
- `Fok` - Fill Or Kill
- `Ioc` - Immediate Or Cancel

#### 5. WebSocket 实时数据 (ws.rs)

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(1000);

// 订阅订单簿更新
tokio::spawn(async move {
    client.ws.subscribe_orderbook(
        vec!["TOKEN1".to_string(), "TOKEN2".to_string()],
        tx
    ).await
});

// 接收更新
while let Some(orderbook) = rx.recv().await {
    println!("Best bid: {:?}", orderbook.best_bid());
}
```

#### 6. 持仓查询 (order.rs)

```rust
// 获取持仓
let positions = client.order.get_positions().await?;

for pos in positions {
    println!("Token: {}, Balance: {}", pos.token_id, pos.balance);
}

// 获取交易记录
let trades = client.order.get_trades(100).await?;
```

## 🔧 在策略中使用

### Market Maker 示例

```rust
use poly_client::{PolyClient, AuthConfig};

// 1. 创建客户端
let auth = AuthConfig::from_private_key(
    &config.private_key,
    &config.clob_host
)?;

let client = PolyClient::with_auth(&config.clob_host, &auth);

// 2. 获取订单簿
let ob = client.get_orderbook(token_id).await?;

// 3. 计算报价
let mid = ob.mid_price().unwrap();
let bid = mid * Decimal::from_str("0.99")?;
let ask = mid * Decimal::from_str("1.01")?;

// 4. 下单
client.order.buy(token_id, bid, size).await?;
client.order.sell(token_id, ask, size).await?;

// 5. 监控成交
let positions = client.order.get_positions().await?;
```

### Arbitrage 示例

```rust
// 扫描市场价差
for market in markets {
    let yes_price = client.get_price(&market.token_ids[0]).await?;
    let no_price = client.get_price(&market.token_ids[1]).await?;
    
    let sum = yes_price + no_price;
    
    if sum < Decimal::ONE - min_profit {
        // 买入套利机会
        client.order.buy(&market.token_ids[0], yes_price, size).await?;
        client.order.buy(&market.token_ids[1], no_price, size).await?;
    }
}
```

### Volatility Hunter 示例

```rust
// WebSocket 实时订阅
let (tx, mut rx) = mpsc::channel(1000);

tokio::spawn(async move {
    client.ws.subscribe_orderbook(token_ids, tx).await
});

// 实时处理
while let Some(ob) = rx.recv().await {
    if let Some(mid) = ob.mid_price() {
        // 检测价格异常
        if is_anomaly(mid) {
            // 快速下单
            client.order.buy(token_id, mid, size).await?;
        }
    }
}
```

## ⚙️ 配置示例

```toml
# config/market-maker.toml

[clob]
host = "https://clob.polymarket.com"

# API 凭证（可选，会自动从私钥派生）
api_key = ""
api_secret = ""

[polygon]
rpc_url = "https://polygon-rpc.com"
private_key = "YOUR_PRIVATE_KEY"  # 用于派生 API 凭证
```

## 🔐 安全注意事项

1. **私钥安全**
   - 不要提交到版本控制
   - 使用环境变量或密钥管理服务
   - 生产环境使用硬件钱包

2. **API 凭证**
   - 定期轮换
   - 限制 IP 访问
   - 监控异常活动

3. **签名安全**
   - 时间戳防重放攻击
   - 请求体哈希防篡改
   - 使用 HTTPS

## 📊 API 限制

| 端点类型 | 限流 | 说明 |
|----------|------|------|
| 公开数据 | 100 req/s | 订单簿、市场数据 |
| 认证端点 | 20 req/s | 下单、撤单 |
| WebSocket | 10 连接/IP | 实时数据 |

## 🐛 错误处理

```rust
use poly_client::types::ApiError;

match client.order.buy(token_id, price, size).await {
    Ok(resp) => {
        println!("Order placed: {}", resp.order_id);
    }
    Err(e) => {
        if let Some(api_err) = e.downcast_ref::<ApiError>() {
            match api_err.code.as_deref() {
                Some("INSUFFICIENT_BALANCE") => {
                    // 余额不足
                }
                Some("INVALID_PRICE") => {
                    // 价格无效
                }
                _ => {
                    // 其他错误
                }
            }
        }
    }
}
```

## 🔍 调试技巧

1. **启用详细日志**
   ```bash
   RUST_LOG=poly_client=debug ./target/release/market-maker
   ```

2. **健康检查**
   ```rust
   if client.health_check().await? {
       println!("API is healthy");
   }
   ```

3. **测试网**
   ```rust
   // 使用测试网
   let client = PolyClient::new("https://testnet-clob.polymarket.com");
   ```

## 📚 参考链接

- [Polymarket API 文档](https://docs.polymarket.com/api-reference)
- [CLOB API 参考](https://docs.polymarket.com/api-reference/clob)
- [认证指南](https://docs.polymarket.com/api-reference/authentication)
