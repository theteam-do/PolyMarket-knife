# 本地 Mock 测试方案

由于无法 fork 主网，我们提供本地 Mock 测试方案。

## 方案 1: Mock 数据测试

### 使用 Mock CLOB 响应

```rust
// 在测试中使用 Mock 数据
use mockall::automock;

#[automock]
#[async_trait]
pub trait ClobClient {
    async fn get_orderbook(&self, token_id: &str) -> Result<OrderBook>;
    async fn place_order(&self, order: Order) -> Result<OrderResponse>;
}

// 测试代码
#[tokio::test]
async fn test_market_maker() {
    let mut mock_client = MockClobClient::new();
    mock_client
        .expect_get_orderbook()
        .returning(|_| Ok(mock_orderbook()));
    
    // 运行测试
    let result = run_strategy(&mock_client).await;
    assert!(result.is_ok());
}
```

### 优势
- ✅ 无需测试币
- ✅ 快速执行
- ✅ 可重复
- ✅ 无网络依赖

## 方案 2: 历史数据回测

### 录制真实 API 响应

```bash
# 录制模式
export RECORD_MODE=true
./target/release/market-maker --config config/test.toml

# 生成 mock 数据
# 保存在 tests/fixtures/orderbook_snapshot.json
```

### 回放测试

```rust
#[tokio::test]
async fn test_with_recorded_data() {
    let fixture = load_fixture("orderbook_snapshot.json");
    let client = ReplayClient::new(fixture);
    
    test_strategy(&client).await;
}
```

## 方案 3: 单元测试

### 测试核心逻辑

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_quotes() {
        let quoter = Quoter::new(config);
        let book = mock_orderbook();
        
        let (bid, ask) = quoter.calculate_quotes(&book);
        
        assert!(bid < ask);
        assert!(bid > 0.0);
    }
    
    #[test]
    fn test_risk_check() {
        let risk = RiskManager::new(config);
        assert!(risk.can_trade());
    }
}
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_calculate_quotes

# 生成覆盖率报告
cargo tarpaulin --out Html
```

## 方案 4: 集成测试框架

### 创建测试工具

```rust
// tests/common/mod.rs
pub struct TestHarness {
    pub client: MockClobClient,
    pub config: Config,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            client: MockClobClient::new(),
            config: test_config(),
        }
    }
    
    pub async fn run_strategy(&mut self) -> Result<()> {
        // 运行完整策略
    }
}
```

### 编写集成测试

```rust
#[tokio::test]
async fn test_full_market_making_cycle() {
    let mut harness = TestHarness::new();
    
    // 模拟市场变化
    harness.client.expect_orderbook().times(10);
    harness.client.expect_place_order().times(5);
    
    // 运行一个完整周期
    harness.run_strategy().await.unwrap();
}
```

## 推荐的测试流程

### 1. 开发阶段
```bash
# 运行单元测试
cargo test --lib

# 快速反馈
cargo test --lib -- --test-threads=1
```

### 2. 集成测试
```bash
# 运行集成测试
cargo test --test '*'

# 使用 Mock 数据
cargo test --features mock
```

### 3. 端到端测试 (可选)
```bash
# 如果有测试网资源
./scripts/test-testnet.sh
```

## Mock 数据示例

### Mock 订单簿

```json
{
  "token_id": "123456",
  "bids": [
    {"price": "0.50", "size": "100"},
    {"price": "0.49", "size": "200"}
  ],
  "asks": [
    {"price": "0.52", "size": "100"},
    {"price": "0.53", "size": "200"}
  ]
}
```

### Mock 订单响应

```json
{
  "order_id": "mock-order-123",
  "status": "success",
  "signature": "0x..."
}
```

## 测试覆盖率目标

| 模块 | 目标覆盖率 | 当前覆盖率 |
|------|-----------|-----------|
| 核心逻辑 | 90%+ | 待测试 |
| 风控模块 | 95%+ | 待测试 |
| API 集成 | 70%+ | 待测试 |
| 工具函数 | 100% | 待测试 |

## 下一步

1. 运行单元测试验证核心逻辑
2. 添加更多 Mock 测试用例
3. 集成测试覆盖完整流程
4. (可选) 获取测试币后进行真实测试

