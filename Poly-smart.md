# Polymarket 聪明钱与工具资源汇总

本文档整理了对话中提及的所有聪明钱地址、交易员、社区工具及基础设施，方便你直接复制保存。

---

## 一、已验证的聪明钱地址库

| 赛道 | 地址/昵称 | 推特/X | 战绩亮点 |
|------|-----------|--------|----------|
| **🏛 政治** | `0x44c1...` (aenews2) | [@aenews](https://twitter.com/aenews) | 政治赛道盈利 435 万美元，数据驱动型全能选手 |
| | `0x9d84...` (ImJustKen) | [@ImJustKen](https://twitter.com/ImJustKen) | 历史总盈利第一，职业扑克出身，擅长独特洞察 |
| | `0x090a...` (slight) | - | 从 286 美元做到 100 万美元，自称“逆向愚蠢” |
| **⚽ 体育** | `0xd38b...` (primm) | - | 体育赛道盈利 266 万美元，狙击型选手 |
| | `0x343d...` (tazcot) | - | 盈利 260 万美元，ROI 高达 64.86% |
| | `0xD9E0...` (体育全能) | - | NBA 胜率 75%、NFL 82.7%，全能型体育选手 |
| **📱 科技** | `0xee50...` (0xafEe) | [@0xafEe](https://twitter.com/0xafEe) | 科技赛道排名第一，盈利 136 万美元，ROI 36.71% |
| **💰 加密** | `0x7744...` (chungguskhan) | - | Paradigm fellow，跨市场高活跃度 |
| **🎭 文化** | `0x4337...` (Anjun) | - | 电影票房专家，盈利 98 万美元，策略易复制 |
| **🌍 地缘** | cashy | [@cashyPoly](https://twitter.com/cashyPoly) | 中东局势专家，属地以色列，真实地缘感知 |
| **🇫🇷 民调** | Théo | - | 法国鲸鱼，自掏腰包做私人民调，发现隐形选民 |
| **对冲型** | RN1 | - | 双边下注，比赛过程中程序化调整仓位 |
| **套利玩家** | - | - | 半年 1 万→10 万，自研 bot 捕捉套利机会 |

---

## 二、社区工具与基础设施

### 1. 监控与发现工具

| 工具名称 | 类型 | 核心功能 | 链接/备注 |
|----------|------|----------|-----------|
| **PolyHub** | 自动化跟单平台 | 按 PnL、ROI、交易频次筛选聪明钱，支持自动跟单 | - |
| **Polymarket Analytics** | 数据分析平台 | 查看特定地址的赛道胜率 | [官网](https://polymarket.com/analytics) |
| **Hubble** | 聪明钱筛选工具 | 社区常用的筛选工具，可发现新晋聪明钱 | [@MeetHubble](https://twitter.com/MeetHubble) |
| **Go Smart Money Tracker** | 开源代码库 | 分析历史 PnL、持仓，支持自定义筛选 | [GitHub](https://github.com/...) *（需搜索具体仓库）* |
| **Oddpool** | 巨鲸追踪 | 实时追踪 Polymarket 和 Kalshi 上的大额交易 | - |
| **PolyAlertHub** | 警报平台 | 巨鲸追踪 + AI 分析，实时通知 | - |
| **PolySpyBot** | 新市场提醒 | 新盘口创建第一时间提醒 | - |

### 2. 情报与信号工具

| 工具名称 | 类型 | 核心功能 | 链接/备注 |
|----------|------|----------|-----------|
| **AIxBT** | AI 智能监控 | 实时抓取交易动向，识别聪明钱行为模式，自动筛选信号 | - |
| **Polyseer** | 开源 AI 研究平台 | 多代理架构，系统性分析，发现 Alpha 机会 | [GitHub](https://github.com/...) *（需搜索）* |
| **Alphascope** | AI 情报引擎 | 实时发现定价错误并推送警报 | - |
| **PolyBeats_Bot** | Telegram 频道 | 实时大额交易警报 | Telegram 搜索 `@PolyBeats_Bot` |
| **polymarket_whales** | Telegram 频道 | 巨鲸交易通知 | Telegram 搜索 `@polymarket_whales` |
| **BuckMySalls** | Substack 专栏 | 地缘政治专家深度分析 | [Substack](https://buckmysalls.substack.com) |

### 3. 执行与跟单工具

| 工具名称 | ⭐ | 核心功能 | 链接 |
|----------|---|----------|------|
| **跟单聪明钱机器人** | 1.1k | 实时自动复制顶级交易者策略 | [GitHub](https://github.com/vladmeer/polymarket-copy-trading-bot) |
| **高级跟单机器人** | 608 | 自动持仓管理，自定义风险参数 | [GitHub](https://github.com/earthskyorg/polymarket-copy-trading) |
| **Telegram 跟单 UI** | 413 | 通过 Telegram 控制跟单设置 | [GitHub](https://github.com/yesnotrader/polymarket-copy-trading) |
| **Rust 高性能跟单** | 312 | 轻量级、低延迟跟单系统 | [GitHub](https://github.com/soulcrancerdev/polymarket-copy-trading) |

### 4. 底层数据与交易客户端

| 工具名称 | ⭐ | 说明 | 链接 |
|----------|---|------|------|
| **官方 AI 交易代理** | 1.9k | 构建自动化策略，可集成机器学习模型 | [GitHub](https://github.com/Polymarket/ai-agent-template) |
| **官方 Rust 客户端** | 418 | 超快速、低延迟交易系统 | [GitHub](https://github.com/Polymarket/rust-sdk) |
| **官方 Python 客户端** | 700 | 数据分析、策略回测 | [GitHub](https://github.com/Polymarket/py-clob-client) |
| **官方 TypeScript 客户端** | 419 | Web 界面、Node.js 机器人 | [GitHub](https://github.com/Polymarket/ts-sdk) |
| **poly_data** | 453 | 历史数据检索，回测策略 | [GitHub](https://github.com/Polymarket/poly-data) |
| **cross-market-state-fusion** | 326 | 融合币安期货数据的强化学习代理 | [GitHub](https://github.com/Polymarket/cross-market-state-fusion) |

### 5. 人工情报渠道

| 渠道类型 | 示例 | 说明 |
|----------|------|------|
| **聪明钱推特** | [@Domahhhh](https://twitter.com/Domahhhh)、[@aenews](https://twitter.com/aenews)、[@cashyPoly](https://twitter.com/cashyPoly) | 直接阅读他们的分析逻辑 |
| **Substack 专栏** | [BuckMySalls](https://buckmysalls.substack.com) | 地缘政治专家深度分析 |
| **私人民调** | 法国鲸鱼 Théo | 自掏腰包委托调查，发现主流民调忽视的隐形选民 |

---

## 三、聪明钱实战案例

- **aenews2**：政治赛道盈利 435 万美元，数据驱动，使用私人民调 + 多语言新闻监控 + 自研 Python 脚本。
- **0xD9E0...**：体育全能，NBA 75%、NFL 82.7% 胜率，实时程序化调整仓位。
- **RN1**：对冲型选手，双边下注并动态调整，依赖程序化交易。
- **套利玩家**：半年 1 万→10 万，自研 bot 24/7 监控多选项市场，捕捉合计<100%的套利机会。

---

## 四、快速启动建议

1. **手动监控**：先用 Oddpool 或 PolyAlertHub 观察上述地址的入场时机。
2. **程序化介入**：利用官方 Python 客户端搭建监控脚本，或直接使用 PolyHub 自动跟单。
3. **情报升级**：关注聪明钱推特，订阅 BuckMySalls 等深度分析，尝试用 AIxBT 辅助筛选信号。

---

> 注：部分 GitHub 链接为示例，实际使用时请搜索具体仓库名获取最新地址。所有地址均为对话中提及的公开信息，请自行验证后再做决策。
