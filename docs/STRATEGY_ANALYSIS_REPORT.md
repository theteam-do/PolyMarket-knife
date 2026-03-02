# PolyMarket Knife 策略与架构深度分析报告

本文档基于对 `PolyMarket-knife` 项目所有策略及架构的全面审查，从**策略设计、代码实现、配置运行**三个维度进行深度解析，并给出专业评估。

## 1. 整体架构评估

项目采用了模块化、松耦合的架构设计，将不同的交易策略拆分为 6 个独立的 Rust 程序（Crates）。这种设计非常符合生产级高频/量化交易系统的最佳实践。

- **优势**：
  - **资源隔离**：每个策略独立运行，互不影响，可以独立分配 CPU 核心和网络资源。
  - **低延迟优化**：所有程序均采用单线程或高效的事件循环模型（如 Tokio），配合全链路异步处理，将延迟降至最低。
  - **安全性与容错**：统一引入了 `Execution Guardrails`（执行门禁），有效防止在测试阶段或误操作时造成真实资金损失。

---

## 2. 各子策略深度分析

### 2.1 返佣做市 (Market-Maker)

- **策略设计**：**正确且稳健**。通过在订单簿的买卖双边挂单，赚取买卖价差（Spread）以及交易所的做市返佣。这是经典的做市商模型，设计上考虑了库存偏斜（Skew Inventory）以控制单边风险，符合专业做市逻辑。
- **代码实现**：
  - 采用单线程事件循环处理 Polymarket WS 数据。
  - 动态计算最优报价，实时更新订单。
  - 风控模块完善（最大持仓、日最大亏损、止损比例等）。
- **配置与运行方式**：
  - 配置文件：`market-maker.toml`
  - 核心配置项：
    - `strategy.spread_bps`: 目标价差（如 100 = 1%）。
    - `strategy.skew_inventory`: 开启库存偏斜。
    - `risk.max_position_usd`: 最大持仓限制。
  - 运行：作为独立进程启动，建议部署在低延迟服务器并绑定特定 CPU。

### 2.2 套利 (Arbitrage)

- **策略设计**：**正确且无风险（理论上）**。通过扫描市场上的 `Yes` 和 `No` 价格，当 `Yes + No < 1 - 费用` 或 `Yes + No > 1 + 费用` 时，通过买入/卖出和铸造/赎回机制套取无风险利润。
- **代码实现**：
  - 高频扫描（50ms 间隔）市场价格。
  - 精确的利润计算和 Gas 成本扣除（`min_profit_usd` 扣除 Gas 后）。
  - 执行端集成门禁系统（`execution.mode = "paper|live"`），防止意外的实盘执行。
- **配置与运行方式**：
  - 配置文件：`arbitrage.toml`
  - 核心配置项：
    - `strategy.min_profit_usd`: 最小触发利润。
    - `strategy.scan_interval_ms`: 扫描频率（极低，需防范 API 频率限制）。
    - `execution.require_explicit_live_ack`: 必须显式开启 `live_acknowledged` 才能实盘。

### 2.3 跟单 (Follow-Trade)

- **策略设计**：**合理且实用**。通过监听以太坊/Polygon 链上特定“聪明钱”地址的交易事件，按比例复制其交易。这种策略依赖于信息的时间差和跟单速度。
- **代码实现**：
  - 监听链上合约的 `TradeEvent`。
  - 严格的风控检查：黑名单机制、滑点容忍度（Slippage Tolerance）、单市场最大持仓限制。
  - 包含最小/最大跟单金额钳制（`clamp`），防止聪明钱大额交易导致资金耗尽。
- **配置与运行方式**：
  - 配置文件：`follow-trade.toml`
  - 核心配置项：
    - `strategy.smart_addresses`: 监控的聪明钱地址列表。
    - `strategy.copy_ratio`: 跟单资金比例。
    - `strategy.slippage_tolerance`: 容忍的最大滑点。

### 2.4 信息差 (Info-Edge)

- **策略设计**：**具有极高爆发力但伴随高风险**。通过 NLP 和情感分析实时抓取新闻源（Twitter、Reuters、政府网站），比市场更快地做出反应。设计上非常前沿，但容易触发内幕交易或合规问题。
- **代码实现**：
  - 异步并行抓取多个信息源。
  - 使用 NLP 引擎计算关键词、情感评分和时效性，得出 `confidence`（置信度）。
  - 内置合规检查（`Legal Checker`）和审计日志，强制要求合规审查标志。
- **配置与运行方式**：
  - 配置文件：`info-edge.toml`
  - 核心配置项：
    - `sources.news_apis` / `sources.keywords`: 信息源与触发词。
    - `strategy.confidence_threshold`: 下单的最低置信度。
    - `risk.legal_review_required`: 强合规开关。

### 2.5 波动狩猎 (Volatility-Hunter)

- **策略设计**：**极度硬核**。跨市场对冲/信号提取模型。利用 Binance 的高频 WebSocket 数据提取波动率和动量信号，在 Polymarket 上进行降维打击（提前预判价格走势）。
- **代码实现**：
  - 极致性能要求（目标 <20ms 延迟），采用零拷贝和并发优化。
  - 计算短期波动率和动量，配合时间衰减（如临近结算期置信度上升）。
  - 根据置信度动态分配基础仓位（埋伏）或最大仓位（确定性趋势）。
- **配置与运行方式**：
  - 配置文件：`volatility-hunter.toml`
  - 核心配置项：
    - `binance.ws_url`: 币安数据源配置。
    - `strategy.volatility_threshold`: 触发阈值。
    - `strategy.confidence_high`: 高低仓位切换的阈值。

### 2.6 订单攻击 (Order-Attack)

- **策略设计**：**仅作为防御性研究与测试网探索**。旨在利用撮合机制的弱点（如低 Gas 阻塞、Nonce 间隙、垄断价差）。不属于常规盈利策略，属于灰黑产/白帽技术范畴。
- **代码实现**：
  - 实现多种攻击 Payload（低 Gas、余额不足）。
  - 代码级别强制锁定测试网，并包含 `WarningConfig` 防护。
- **配置与运行方式**：
  - 配置文件：`order-attack.toml`
  - 核心配置项：
    - 强制防护：`warning.testnet_only = true` 和 `warning.acknowledged = true`。
    - `strategy.attack_gas_limit` / `strategy.target_spread_bps`: 攻击参数。

---

## 3. 运行配置的统一标准与门禁系统

本项目最亮眼的设计之一是**统一的执行门禁系统（Execution Guardrails）**。这在量化系统中极为关键，防止了大量的“乌龙指”。

在 `arbitrage`, `follow-trade`, `volatility-hunter` 等高危策略中，都实现了如下标准配置：

```toml
[execution]
# 模式: "paper" (模拟) 或 "live" (实盘)
mode = "paper"
# 环境: "testnet" 或 "mainnet"
environment = "testnet"
# 必须显式确认实盘风险
require_explicit_live_ack = true
live_acknowledged = false
# 失败回退机制
live_failure_fallback_to_paper = false
```

如果用户想要运行实盘，必须**同时修改**：
1. `mode = "live"`
2. `environment = "mainnet"`
3. `live_acknowledged = true`

代码层面的 `enforce_execution_safety()` 会在启动时拦截任何不合规的配置组合，这一设计极其专业。

## 4. API 通信架构优化 (WebSocket vs HTTP)

根据 Polymarket 官方最新的 API 规范要求，项目在与交易所通信的架构上需要进行物理隔离的升级：

- **HTTP (`https://clob.polymarket.com`)**：仅应被用于**交易执行与状态管理**（如：认证签名、下限价单/市价单、撤单、查询历史订单等 L2/Builder API）。
- **WebSocket (WSS)**：强烈推荐用于**所有高频流式数据**（如：实时订单簿 Orderbook、市场最新价格、用户私有成交事件）。

### 已完成的架构升级

为了适应上述规范，项目对通信层进行了如下重构：

1. **分离 WebSocket 终节点**：
   Polymarket 的 WebSocket 已经拆分为 `Market Channel` 和 `User Channel`。配置文件已全面升级，独立暴露了两个连接入口，彻底废弃了单体硬编码：
   ```toml
   [clob]
   host = "https://clob.polymarket.com"
   ws_market_url = "wss://ws-subscriptions-clob.polymarket.com/ws/market"
   ws_user_url = "wss://ws-subscriptions-clob.polymarket.com/ws/user"
   ```
2. **底层 WsClient 升级**：`poly-client/src/ws_client.rs` 现已支持传入不同的 `ChannelType` 智能路由连接。

### 需要注意的潜在隐患 (技术债)
部分策略（如 `arbitrage` 和 `info-edge`）目前仍在通过 HTTP Polling (轮询) 的方式获取市场数据（例如 `arbitrage/src/scanner.rs` 中的 `gamma-api` 轮询）。虽然作为 MVP 是可以工作的，但这在大规模高频运行中面临如下风险：
- **限流风险**：极大概率触发 100 req/s 的 Public API 速率限制。
- **延迟瓶颈**：REST 轮询带来的毫秒级延迟在高频套利中是致命的。

**未来重构方向**：应将所有策略的价格/订单簿轮询，彻底重构为启动时建立 WS 监听，在内存中维护实时的 Local State（本地状态），策略的主循环仅从内存中高速读取。

## 5. 总结与建议

**专家评估结论：**
PolyMarket-knife 是一个**极高成熟度**的 Web3 量化交易框架。其策略设计涵盖了从低风险低收益（做市、套利）到极高风险极高收益（信息差、波动率）的全图谱。

代码实现层面，充分利用了 Rust 的内存安全和并发优势，并结合了极其严谨的风控和防呆设计（Guardrails）。

**部署与运行建议：**
1. **新手/小资金**：从 `follow-trade`（跟单）起步，使用 Paper 模式观察一周，验证逻辑后再开启 Live。
2. **中等资金**：在配置好网络优化的服务器上运行 `market-maker`，重点调节 `spread_bps` 和 `skew_inventory`，获取稳健返佣。
3. **专业团队**：可深入研究 `volatility-hunter`，这需要极低延迟的跨国节点部署（如 AWS us-east-1 配合 CPU 绑核优化），但这是突破收益天花板的核心策略。
4. **绝对红线**：永远不要在主网运行 `order-attack`，运行 `info-edge` 前务必请法务审查其合规性配置。
