# 迁移到官方 Polymarket SDK 详细计划

**分支**: `feature/migrate-to-official-sdk`  
**创建日期**: 2026-03-01  
**预计工期**: 7-11 天

---

## 📋 目录

1. [迁移目标](#迁移目标)
2. [迁移范围](#迁移范围)
3. [依赖变更](#依赖变更)
4. [分阶段计划](#分阶段计划)
5. [详细实施步骤](#详细实施步骤)
6. [测试计划](#测试计划)
7. [回滚方案](#回滚方案)
8. [验收标准](#验收标准)

---

## 🎯 迁移目标

### 核心目标

- ✅ 将所有策略的底层 API 调用从 `poly-client` 迁移到 `polymarket-client-sdk` (官方)
- ✅ 保持所有策略功能不变
- ✅ 保持现有策略逻辑和风控模块
- ✅ 通过所有单元测试和集成测试

### 非目标

- ❌ 不改变策略核心逻辑
- ❌ 不改变配置文件结构
- ❌ 不改变日志和监控系统

---

## 📦 迁移范围

### 需要迁移的模块

| 模块 | 当前实现 | 迁移到 | 工作量 |
|------|----------|--------|--------|
| **poly-client** | 自研 API 客户端 | 官方 SDK | 高 |
| **market-maker** | 使用 poly-client | 官方 SDK | 中 |
| **arbitrage** | 使用 poly-client | 官方 SDK | 中 |
| **follow-trade** | 使用 poly-client | 官方 SDK | 中 |
| **volatility-hunter** | 使用 poly-client | 官方 SDK | 中 |
| **info-edge** | 使用 poly-client | 官方 SDK | 低 |
| **order-attack** | 使用 poly-client | 官方 SDK | 低 |

### 保持不变的模块

- ✅ 所有策略逻辑 (`strategy/`)
- ✅ 风控模块 (`risk/`)
- ✅ 配置文件 (`config/`)
- ✅ 日志系统 (`tracing`)
- ✅ 监控指标

---

## 🔧 依赖变更

### Cargo.toml 变更

#### 修改前
```toml
[workspace.dependencies]
ethers = { version = "2.0", features = ["ws", "abigen"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio-tungstenite = { version = "0.26", features = ["native-tls"] }
```

#### 修改后
```toml
[workspace.dependencies]
# 官方 SDK
polymarket-client-sdk = { version = "0.4", features = ["clob", "data", "gamma", "ws", "ctf"] }

# 保留的依赖
tokio = { version = "1.43", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
rust_decimal = "1.36"

# 官方 SDK 已包含的依赖（移除重复）
# - reqwest
# - tokio-tungstenite
# - alloy (替代 ethers)
```

### 代码层面变更

#### 认证模块
```rust
// 修改前 (poly-client)
use poly_client::AuthConfig;
let auth = AuthConfig::from_private_key(private_key, host)?;

// 修改后 (官方 SDK)
use polymarket_client_sdk::auth::{Credentials, LocalSigner};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
let signer = LocalSigner::from_str(private_key)?.with_chain_id(Some(POLYGON));
let client = Client::new(host, Config::default())?
    .authentication_builder(&signer)
    .authenticate()
    .await?;
```

#### 订单簿查询
```rust
// 修改前
use poly_client::PolyClient;
let ob = client.get_orderbook(token_id).await?;

// 修改后
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::request::OrderBookRequest;
let ob = client.orderbook(&OrderBookRequest {
    token_id: U256::from_str(token_id)?,
}).await?;
```

#### 下单
```rust
// 修改前
client.order.buy(token_id, price, size).await?;

// 修改后
use polymarket_client_sdk::clob::order_builder::LimitParams;
let order = builder.limit(LimitParams {
    token_id: U256::from_str(token_id)?,
    price: Decimal::from_f64_retain(price)?,
    size: Decimal::from_f64_retain(size)?,
    side: Side::Buy,
    fee_rate_bps: 0,
    nonce: generate_nonce(),
}).await?;
client.create_order(order).await?;
```

---

## 📅 分阶段计划

### 第一阶段：准备工作 (1 天)

**目标**：完成依赖更新和基础框架搭建

**任务**：
- [ ] 更新 workspace Cargo.toml
- [ ] 创建官方 SDK 封装模块
- [ ] 实现认证适配层
- [ ] 编写迁移指南文档

**验收标准**：
- ✅ 项目可以编译通过
- ✅ 官方 SDK 可以正常初始化
- ✅ 认证流程测试通过

---

### 第二阶段：核心模块迁移 (2-3 天)

**目标**：完成 poly-client 到官方 SDK 的迁移

**任务**：
- [ ] 创建 `poly-adapter` 适配层
- [ ] 实现订单簿查询适配
- [ ] 实现下单/撤单适配
- [ ] 实现 WebSocket 适配
- [ ] 实现 Data API 适配

**验收标准**：
- ✅ 所有基础 API 有对应适配
- ✅ 类型转换正确
- ✅ 错误处理完善

---

### 第三阶段：策略迁移 (3-4 天)

**目标**：逐个迁移 6 个策略

#### Day 1-2: market-maker + arbitrage
- [ ] market-maker 策略迁移
- [ ] arbitrage 策略迁移
- [ ] 单元测试修复

#### Day 3: follow-trade + volatility-hunter
- [ ] follow-trade 策略迁移
- [ ] volatility-hunter 策略迁移
- [ ] 单元测试修复

#### Day 4: info-edge + order-attack
- [ ] info-edge 策略迁移
- [ ] order-attack 策略迁移
- [ ] 单元测试修复

**验收标准**：
- ✅ 所有策略编译通过
- ✅ 所有单元测试通过
- ✅ 策略逻辑无变化

---

### 第四阶段：测试验证 (2 天)

**目标**：全面测试确保功能正常

**任务**：
- [ ] 运行所有单元测试
- [ ] 运行集成测试
- [ ] 测试网验证（可选）
- [ ] 性能基准测试

**验收标准**：
- ✅ 单元测试通过率 100%
- ✅ 集成测试通过率 100%
- ✅ 性能无明显下降

---

### 第五阶段：清理优化 (1 天)

**目标**：清理旧代码，优化结构

**任务**：
- [ ] 移除旧的 poly-client 代码
- [ ] 更新文档
- [ ] 代码审查
- [ ] 性能优化

**验收标准**：
- ✅ 无死代码
- ✅ 文档更新完成
- ✅ 代码审查通过

---

## 🔍 详细实施步骤

### 步骤 1: 更新依赖

```bash
# 1. 编辑 Cargo.toml
vim Cargo.toml

# 2. 添加官方 SDK
[workspace.dependencies]
polymarket-client-sdk = { version = "0.4", features = ["clob", "data", "gamma", "ws", "ctf"] }

# 3. 移除冲突依赖
# - ethers (替换为 alloy)
# - reqwest (官方 SDK 已包含)
# - tokio-tungstenite (官方 SDK 已包含)

# 4. 更新依赖
cargo update
```

### 步骤 2: 创建适配层

创建 `src/adapter/` 目录：

```
src/adapter/
├── mod.rs           # 模块入口
├── auth.rs          # 认证适配
├── clob.rs          # CLOB API 适配
├── data.rs          # Data API 适配
├── types.rs         # 类型转换
└── error.rs         # 错误转换
```

### 步骤 3: 类型转换

```rust
// src/adapter/types.rs

use polymarket_client_sdk::types::{Address as SdkAddress, U256 as SdkU256};
use rust_decimal::Decimal as SdkDecimal;

// 类型别名，保持代码一致性
pub type Address = SdkAddress;
pub type TokenId = SdkU256;
pub type Price = SdkDecimal;

// 转换函数
pub fn str_to_token_id(s: &str) -> Result<TokenId> {
    Ok(TokenId::from_str(s)?)
}

pub fn f64_to_decimal(f: f64) -> Result<Price> {
    Ok(Price::from_f64_retain(f)?)
}
```

### 步骤 4: 认证适配

```rust
// src/adapter/auth.rs

use polymarket_client_sdk::auth::{Credentials, LocalSigner};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};

pub struct AuthAdapter {
    signer: LocalSigner,
    host: String,
}

impl AuthAdapter {
    pub fn new(private_key: &str, host: &str) -> Result<Self> {
        let signer = LocalSigner::from_str(private_key)?
            .with_chain_id(Some(POLYGON));
        Ok(Self {
            signer,
            host: host.to_string(),
        })
    }

    pub async fn create_client(&self) -> Result<Client> {
        let client = Client::new(&self.host, Config::default())?
            .authentication_builder(&self.signer)
            .authenticate()
            .await?;
        Ok(client)
    }
}
```

### 步骤 5: 策略代码迁移示例

#### market-maker 迁移

```rust
// 修改前
use poly_client::{PolyClient, OrderBook};

pub struct Executor {
    client: PolyClient,
}

impl Executor {
    pub async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        self.client.get_orderbook(token_id).await
    }
}

// 修改后
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::request::OrderBookRequest;
use crate::adapter::types::{TokenId, str_to_token_id};

pub struct Executor {
    client: Client,
}

impl Executor {
    pub async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        let token_id = str_to_token_id(token_id)?;
        let request = OrderBookRequest { token_id };
        self.client.orderbook(&request).await
    }
}
```

---

## 🧪 测试计划

### 单元测试

```bash
# 运行所有单元测试
cargo test --lib

# 运行特定模块测试
cargo test -p market-maker
cargo test -p arbitrage

# 显示测试输出
cargo test -- --nocapture
```

### 集成测试

```bash
# 运行集成测试
cargo test --test '*'

# 测试网验证（可选）
# 配置测试网
export CLOB_HOST=https://testnet-clob.polymarket.com
cargo test --test integration -- --ignored
```

### 性能测试

```bash
# 运行基准测试
cargo bench

# 对比迁移前后性能
# 关注指标：延迟、吞吐量、内存使用
```

---

## 🔄 回滚方案

### 如果迁移失败

```bash
# 1. 切换回 main 分支
git checkout main

# 2. 删除迁移分支
git branch -D feature/migrate-to-official-sdk

# 3. 重新基于 main 创建功能分支
git checkout -b feature/new-approach
```

### 保留旧代码

- ✅ 在迁移完成前，不删除 `poly-client` 目录
- ✅ 使用 git tag 标记迁移前状态
- ✅ 保留完整的提交历史

---

## ✅ 验收标准

### 功能验收

- [ ] 所有 6 个策略可以正常编译
- [ ] 所有单元测试通过（100%）
- [ ] 所有集成测试通过（100%）
- [ ] 策略逻辑无变化
- [ ] 配置文件兼容

### 性能验收

- [ ] 延迟无明显增加（<10%）
- [ ] 内存使用无明显增加（<20%）
- [ ] CPU 使用率无明显增加（<10%）

### 代码质量

- [ ] 通过 `cargo clippy` 检查
- [ ] 通过 `cargo fmt` 格式化
- [ ] 无 `TODO` 注释
- [ ] 文档更新完成

### 文档验收

- [ ] README.md 更新
- [ ] API 文档更新
- [ ] 迁移指南完成
- [ ] 示例代码更新

---

## 📊 进度跟踪

### 甘特图

```
Week 1:
Mon-Tue: 准备工作 [████████] 100%
Wed-Fri: 核心模块 [████████] 100%

Week 2:
Mon-Tue: market-maker + arbitrage [████████] 100%
Wed-Thu: follow-trade + volatility [████████] 100%
Fri: info-edge + order-attack [████████] 100%

Week 3:
Mon-Tue: 测试验证 [████████] 100%
Wed: 清理优化 [████████] 100%
```

### 状态标记

- 🔴 未开始
- 🟡 进行中
- 🟢 已完成
- ⚠️ 有风险

---

## 📞 联系方式

**问题反馈**:
- GitHub Issues: https://github.com/theteam-do/PolyMarket-knife/issues
- Email: developer@polymarket-knife.dev

**最后更新**: 2026-03-01
