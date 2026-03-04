# 错误处理指南

本文档介绍 PolyMarket Knife 项目的错误处理最佳实践。

---

## 目录

1. [错误类型选择](#错误类型选择)
2. [错误传播](#错误传播)
3. [错误转换](#错误转换)
4. [错误恢复](#错误恢复)
5. [错误日志](#错误日志)
6. [常见模式](#常见模式)

---

## 错误类型选择

### 库代码：使用自定义错误类型

```rust
// common/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("配置错误：{0}")]
    Config(String),
    
    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),
    
    #[error("解析错误：{0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, CommonError>;
```

**优点**:
- 类型安全
- 清晰的错误层次
- 易于测试

### 应用代码：使用 anyhow

```rust
// market-maker/src/main.rs
use anyhow::{Context, Result};

async fn main() -> Result<()> {
    let config = load_config("config.toml")
        .await
        .context("Failed to load configuration")?;
    
    Ok(())
}
```

**优点**:
- 简洁
- 自动错误链
- 适合应用层

---

## 错误传播

### 使用 `?` 操作符

```rust
// ✅ 推荐：简洁的错误传播
async fn fetch_orderbook(token_id: &str) -> Result<OrderBook> {
    let response = client.get_orderbook(token_id).await?;
    let orderbook = parse_orderbook(response)?;
    Ok(orderbook)
}
```

### 添加上下文

```rust
// ✅ 推荐：提供有意义的错误信息
async fn load_config(path: &str) -> Result<Config> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read config file: {}", path))?;
    
    let config: Config = toml::from_str(&content)
        .context("Failed to parse TOML configuration")?;
    
    Ok(config)
}
```

### 避免过度包装

```rust
// ❌ 不推荐：重复的错误信息
async fn process() -> Result<()> {
    let data = fetch_data()
        .await
        .context("Failed to fetch data")?;
    
    let result = process_data(data)
        .await
        .context("Failed to process data")?; // 冗余
    
    Ok(())
}

// ✅ 推荐：只在边界添加上下文
async fn process() -> Result<()> {
    let data = fetch_data().await?;
    let result = process_data(data).await?;
    Ok(())
}
```

---

## 错误转换

### From trait

```rust
// ✅ 自动转换外部错误
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误：{0}")]
    Database(#[from] sqlx::Error),
    
    #[error("HTTP 错误：{0}")]
    Http(#[from] reqwest::Error),
}
```

### map_err

```rust
// ✅ 手动转换错误类型
let value = parse_input(input)
    .map_err(|e| AppError::Parse(e.to_string()))?;
```

### 使用 anyhow::bail

```rust
// ✅ 快速返回错误
if balance < required {
    anyhow::bail!("Insufficient balance: {} < {}", balance, required);
}
```

---

## 错误恢复

### 提供默认值

```rust
// ✅ 使用 unwrap_or 提供默认值
let timeout = config.timeout.unwrap_or(Duration::from_secs(30));

// ✅ 使用 unwrap_or_else 惰性计算默认值
let config = load_config()
    .await
    .unwrap_or_else(|_| Config::default());
```

### 降级处理

```rust
// ✅ 优雅降级
match fetch_from_primary().await {
    Ok(data) => data,
    Err(e) => {
        tracing::warn!("Primary source failed: {}, using cache", e);
        fetch_from_cache().await?
    }
}
```

### 重试机制

```rust
// ✅ 实现重试逻辑
use tokio::time::{sleep, Duration};

async fn with_retry<F, T>(mut operation: F, max_retries: u32) -> Result<T>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T>>,
{
    let mut attempts = 0;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                let delay = Duration::from_secs(2u64.pow(attempts));
                tracing::warn!("Attempt {} failed: {}. Retrying in {:?}...", attempts, e, delay);
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 错误日志

### 结构化日志

```rust
// ✅ 使用 tracing 记录错误
use tracing::{error, warn, info};

match process_order(order).await {
    Ok(id) => info!(order_id = %id, "Order processed successfully"),
    Err(e) => error!(
        error = %e,
        order_id = %order.id,
        user_id = %order.user_id,
        "Failed to process order"
    ),
}
```

### 错误链

```rust
// ✅ 记录完整错误链
use anyhow::Context;

if let Err(e) = start_service().await {
    tracing::error!("Service startup failed: {:?}", e); // 使用 {:?} 打印完整链
}
```

### 错误分类

```rust
// ✅ 按严重程度分类日志
match error {
    Error::Network(e) => warn!("Network issue (recoverable): {}", e),
    Error::Config(e) => error!("Configuration error (fatal): {}", e),
    Error::Validation(e) => info!("Validation failed (user error): {}", e),
}
```

---

## 常见模式

### Result 类型别名

```rust
// ✅ 在模块级别定义 Result 别名
pub type Result<T> = std::result::Result<T, AppError>;

// 使用
async fn fetch_data() -> Result<Data> {
    // ...
}
```

### Option 到 Result

```rust
// ✅ 转换 Option 为 Result
let user = get_user(id)
    .await
    .ok_or_else(|| AppError::UserNotFound(id))?;
```

### 多错误收集

```rust
// ✅ 收集多个错误而不是立即返回
let mut errors = Vec::new();

for item in items {
    if let Err(e) = process(item).await {
        errors.push(e);
    }
}

if !errors.is_empty() {
    return Err(AppError::MultipleErrors(errors));
}
```

### 错误报告

```rust
// ✅ 生成错误报告
use anyhow::Context;

async fn run() -> anyhow::Result<()> {
    init_config()
        .await
        .context("Configuration initialization failed")?;
    
    start_services()
        .await
        .context("Service startup failed")?;
    
    Ok(())
}

// 在 main 中打印报告
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:?}", e); // 使用 {:?} 打印完整链
        std::process::exit(1);
    }
}
```

---

## 反模式

### ❌ 滥用 unwrap

```rust
// ❌ 避免：可能 panic
let config = load_config().unwrap();

// ✅ 推荐：适当处理
let config = load_config().await?;
```

### ❌ 忽略错误

```rust
// ❌ 避免：错误被静默忽略
let _ = save_data(data);

// ✅ 推荐：至少记录警告
if let Err(e) = save_data(data).await {
    tracing::warn!("Failed to save data: {}", e);
}
```

### ❌ 过度包装

```rust
// ❌ 避免：多层无意义的包装
do_something()
    .map_err(|e| Error::Step1(e))?;
    .map_err(|e| Error::Step2(e))?;
    .map_err(|e| Error::Step3(e))?;

// ✅ 推荐：在边界包装
do_something().context("Operation failed")?;
```

---

## 测试错误处理

### 测试错误情况

```rust
#[test]
fn test_invalid_config() {
    let config = Config::load("nonexistent.toml");
    assert!(config.is_err());
    assert!(config.unwrap_err().to_string().contains("No such file"));
}
```

### 测试错误转换

```rust
#[test]
fn test_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app_error: AppError = io_error.into();
    
    assert!(matches!(app_error, AppError::Io(_)));
}
```

---

## 参考资源

- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror crate](https://github.com/dtolnay/thiserror)
- [anyhow crate](https://github.com/dtolnay/anyhow)
- [tracing crate](https://github.com/tokio-rs/tracing)

---

**最后更新**: 2026-03-05  
**维护者**: PolyMarket Knife Team
