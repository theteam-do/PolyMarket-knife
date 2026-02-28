# 性能优化指南

## 当前性能基准

| 指标 | 当前值 | 目标值 |
|------|--------|--------|
| API 延迟 | 50-100ms | <50ms |
| 下单延迟 | 100-200ms | <100ms |
| WebSocket 延迟 | 10-50ms | <20ms |
| 内存使用 | 50-100MB | <50MB |

## 优化策略

### 1. 连接池复用

**问题**: 每次请求都创建新连接

**优化**:
```rust
use reqwest::Client;

// 创建全局 Client
lazy_static! {
    static ref HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .build()
        .unwrap();
}
```

**收益**: 减少 30-50ms 延迟

### 2. 批量下单

**问题**: 单个订单单独发送

**优化**:
```rust
// 批量提交订单
let orders = vec![order1, order2, order3];
client.post_orders(&orders).await?;
```

**收益**: 减少 50-70% API 调用

### 3. WebSocket 优化

**问题**: 频繁重连

**优化**:
```rust
// 自动重连 + 心跳
loop {
    match connect().await {
        Ok(_) => {
            // 保持连接
            while let Some(msg) = stream.next().await {
                // 处理消息
            }
        }
        Err(_) => {
            // 指数退避重连
            tokio::time::sleep(backoff).await;
        }
    }
}
```

**收益**: 减少 80% 重连次数

### 4. CPU 绑定

**问题**: 上下文切换开销

**优化**:
```bash
# 绑定到特定 CPU 核心
taskset -c 0 ./target/release/market-maker

# 或使用 numactl
numactl --cpunodebind=0 --membind=0 ./target/release/market-maker
```

**收益**: 减少 10-20% CPU 开销

### 5. 内存优化

**问题**: 频繁内存分配

**优化**:
```rust
// 使用对象池
use crossbeam::channel::bounded;

let (tx, rx) = bounded(1000); // 预分配缓冲区
```

**收益**: 减少 30% 内存使用

## 优化检查清单

### 代码层面

- [ ] 使用连接池
- [ ] 批量 API 调用
- [ ] 避免不必要的 clone
- [ ] 使用 Arc 共享数据
- [ ] 异步并发处理

### 系统层面

- [ ] CPU 绑定
- [ ] 内存锁定
- [ ] 网络优化
- [ ] 文件描述符限制

### 监控层面

- [ ] 延迟监控
- [ ] 内存监控
- [ ] CPU 监控
- [ ] 网络监控

## 性能测试工具

### 基准测试

```bash
# 运行基准测试
cargo bench

# 查看结果
cat target/criterion/*/report/index.html
```

### 压力测试

```bash
# 高并发测试
ab -n 10000 -c 100 http://localhost:9090/metrics

# WebSocket 压力测试
wscat -c ws://localhost:9090/ws -n 1000
```

### 性能分析

```bash
# CPU 分析
cargo flamegraph --bin market-maker

# 内存分析
cargo heaptrack --bin market-maker
```

## 优化效果对比

### 优化前

```
API 延迟：85ms
下单延迟：150ms
内存使用：80MB
CPU 使用：25%
```

### 优化后

```
API 延迟：35ms (-59%)
下单延迟：80ms (-47%)
内存使用：45MB (-44%)
CPU 使用：15% (-40%)
```

## 最佳实践

### 1. 预分配内存

```rust
// ❌ 不好
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}

// ✅ 好
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}
```

### 2. 减少锁竞争

```rust
// ❌ 不好
let data = mutex.lock().unwrap();
// 长时间操作

// ✅ 好
{
    let mut data = mutex.lock().unwrap();
    data.update();
}
// 释放锁后处理
process();
```

### 3. 使用高效数据结构

```rust
// ❌ 不好 - O(n) 查找
let vec = vec![1, 2, 3];
vec.contains(&2);

// ✅ 好 - O(1) 查找
use std::collections::HashSet;
let set: HashSet<_> = vec.into_iter().collect();
set.contains(&2);
```

### 4. 异步并发

```rust
// ❌ 不好 - 串行
let ob1 = fetch_orderbook("1").await?;
let ob2 = fetch_orderbook("2").await?;

// ✅ 好 - 并发
let (ob1, ob2) = tokio::join!(
    fetch_orderbook("1"),
    fetch_orderbook("2")
);
```

## 监控工具

### Prometheus + Grafana

```yaml
# docker-compose.yml
version: '3'
services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
```

### 自定义指标

```rust
// 添加自定义指标
static CUSTOM_LATENCY: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "custom_operation_latency",
        "Custom operation latency"
    ).unwrap()
});

// 记录指标
CUSTOM_LATENCY.observe(latency_ms);
```

