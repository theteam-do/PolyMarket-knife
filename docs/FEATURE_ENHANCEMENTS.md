# 功能增强计划

## 1. WebSocket 实时数据

### 当前状态
- ✅ 基础 WebSocket 连接
- ⚠️ 自动重连待优化
- ⚠️ 心跳机制待实现

### 实现方案

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

pub struct WsClient {
    url: String,
    reconnect_delay: Duration,
    max_reconnects: u32,
}

impl WsClient {
    pub async fn connect(&self) -> Result<WebSocketStream> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        Ok(ws_stream)
    }
    
    pub async fn stream_with_reconnect(
        &self,
        tx: mpsc::Sender<Message>,
    ) -> Result<()> {
        let mut reconnects = 0;
        
        loop {
            match self.connect_and_stream(&tx).await {
                Ok(()) => {
                    info!("WebSocket disconnected");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                }
            }
            
            reconnects += 1;
            if reconnects >= self.max_reconnects {
                return Err(anyhow::anyhow!("Max reconnects reached"));
            }
            
            let delay = self.reconnect_delay * (2_u32.pow(reconnects));
            info!("Reconnecting in {}s...", delay.as_secs());
            tokio::time::sleep(delay).await;
        }
    }
}
```

### 收益
- ✅ 实时订单簿更新 (<10ms 延迟)
- ✅ 自动重连 (指数退避)
- ✅ 心跳保活 (30s 间隔)

---

## 2. 批量下单优化

### 当前状态
- ❌ 单个订单单独提交
- ❌ 无批量 API 支持

### 实现方案

```rust
pub struct BatchOrderClient {
    client: Client,
    batch_size: usize,
    queue: Vec<OrderRequest>,
}

impl BatchOrderClient {
    pub fn new(client: Client, batch_size: usize) -> Self {
        Self {
            client,
            batch_size,
            queue: Vec::with_capacity(batch_size),
        }
    }
    
    pub async fn submit_order(&mut self, order: OrderRequest) -> Result<()> {
        self.queue.push(order);
        
        if self.queue.len() >= self.batch_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    pub async fn flush(&mut self) -> Result<Vec<OrderResponse>> {
        if self.queue.is_empty() {
            return Ok(vec![]);
        }
        
        // 批量提交
        let orders = std::mem::take(&mut self.queue);
        let responses = self.client.post_orders(&orders).await?;
        
        Ok(responses)
    }
}
```

### 收益
- ✅ 减少 50-70% API 调用
- ✅ 降低延迟 (批量处理)
- ✅ 提高吞吐量

---

## 3. Telegram 告警通知

### 实现方案

```rust
use reqwest::Client;

pub struct TelegramBot {
    client: Client,
    bot_token: String,
    chat_id: String,
}

impl TelegramBot {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            client: Client::new(),
            bot_token,
            chat_id,
        }
    }
    
    pub async fn send_alert(&self, message: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );
        
        let payload = serde_json::json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "Markdown"
        });
        
        self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        Ok(())
    }
}
```

### 告警模板

```markdown
🚨 **交易告警**

策略：Market Maker
时间：2026-03-01 10:30:00
级别：⚠️ Warning

详情：
- 日亏损：$450 (阈值：$500)
- 当前持仓：$8,000
- 延迟：120ms

建议操作：检查市场波动
```

---

## 4. 回测框架

### 实现方案

```rust
pub struct Backtester {
    strategy: Box<dyn Strategy>,
    data: Vec<MarketData>,
    initial_capital: Decimal,
}

impl Backtester {
    pub fn new(strategy: Box<dyn Strategy>, data: Vec<MarketData>) -> Self {
        Self {
            strategy,
            data,
            initial_capital: dec!(10000),
        }
    }
    
    pub async fn run(&mut self) -> Result<BacktestResult> {
        let mut capital = self.initial_capital;
        let mut trades = Vec::new();
        
        for market_data in &self.data {
            // 运行策略
            let signals = self.strategy.generate_signals(market_data).await?;
            
            // 执行交易
            for signal in signals {
                let trade = self.execute_signal(&signal, &mut capital).await?;
                trades.push(trade);
            }
        }
        
        Ok(BacktestResult {
            initial_capital: self.initial_capital,
            final_capital: capital,
            trades,
            sharpe_ratio: self.calculate_sharpe_ratio(&trades),
            max_drawdown: self.calculate_max_drawdown(&trades),
        })
    }
}
```

### 回测指标

- 总收益率
- 夏普比率
- 最大回撤
- 胜率
- 盈亏比
- 交易次数

---

## 实施优先级

1. **WebSocket 实时数据** ⭐⭐⭐⭐⭐
   - 预计时间：1 天
   - 收益：延迟降低 80%

2. **批量下单优化** ⭐⭐⭐⭐
   - 预计时间：1 天
   - 收益：API 调用减少 70%

3. **Telegram 告警** ⭐⭐⭐
   - 预计时间：0.5 天
   - 收益：及时通知

4. **回测框架** ⭐⭐
   - 预计时间：2-3 天
   - 收益：策略验证

---

