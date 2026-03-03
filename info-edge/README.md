# Info Edge - 信息差交易策略

## ⚠️ 法律警告

**本策略可能涉及内幕交易法律风险。仅供学习研究，请勿用于实际交易。**

已有案例：以色列军人因使用内幕信息在 Polymarket 交易被起诉。

## 🎯 策略核心

比市场更早知道重大事件结果，在价格未反应前建仓。

## 📊 信息渠道分类

### 1. 公开另类数据（合法）
| 数据源 | 监控内容 | 适用市场 |
|--------|----------|----------|
| 政府网站 | 政策更新、任命公告 | 政治市场 |
| 航空公司 API | 专机飞行轨迹 | 国际事件 |
| 电力数据 | 特定区域用电量 | 经济数据 |
| 社交媒体 | 异常活动模式 | 各类事件 |

### 2. 灰色渠道（高风险）
- 内部人士消息
- 未公开报告
- 提前获取的数据

**⚠️ 使用灰色渠道可能构成内幕交易罪**

## ⚡ 性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 新闻获取延迟 | <100ms | API 响应时间 |
| NLP 处理延迟 | <50ms | 情感分析时间 |
| 下单延迟 | <200ms | 信号到执行 |

## 🔧 配置示例

```toml
# config/info-edge.toml
[polygon]
rpc_url = "wss://polygon-bor-rpc.publicnode.com"
private_key = "0x..."

[sources]
# 新闻源
news_apis = [
    { name = "twitter", url = "https://api.twitter.com/2/tweets/search/recent", token = "..." },
    { name = "reuters", url = "https://api.reuters.com/news", token = "..." },
]

# 监控关键词
keywords = [
    "election",
    "fed rate",
    "gdp",
    "unemployment",
]

# 政府网站监控
gov_websites = [
    "https://www.whitehouse.gov/briefing-room/",
    "https://www.federalreserve.gov/newsevents.htm",
]

[strategy]
confidence_threshold = 0.8    # 置信度阈值
max_position_usd = 50000
min_expected_return = 0.3     # 最小预期收益 30%

[risk]
max_daily_loss = 5000
legal_review_required = true  # 强制法律审查
```

## 🏗️ 架构设计

```
┌────────────────────────────────────────────────────────┐
│                  News Collector                         │
│         (并行抓取多个新闻源)                             │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│                   NLP Engine                            │
│    (情感分析、关键词匹配、置信度计算)                    │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│                Signal Generator                         │
│         (生成交易信号，计算预期收益)                     │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│               Legal Checker ⚠️                          │
│         (合规检查，记录审计日志)                         │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│                  Executor                               │
│              (执行交易)                                  │
└────────────────────────────────────────────────────────┘
```

## 📝 核心逻辑

```rust
// 1. 新闻抓取
async fn fetch_news(&self) -> Vec<NewsItem> {
    let tasks: Vec<_> = self.sources.iter()
        .map(|source| self.fetch_from_source(source))
        .collect();
    
    future::join_all(tasks)
        .await
        .into_iter()
        .flatten()
        .collect()
}

// 2. NLP 分析
fn analyze_sentiment(&self, text: &str) -> SentimentResult {
    // 关键词匹配
    let keyword_score = self.match_keywords(text);
    
    // 情感分析
    let sentiment_score = self.sentiment_model.predict(text);
    
    // 时效性评分
    let recency_score = self.calc_recency_score(text);
    
    SentimentResult {
        confidence: (keyword_score + sentiment_score + recency_score) / 3.0,
        direction: if sentiment_score > 0.0 { Direction::Yes } else { Direction::No },
    }
}

// 3. 信号生成
fn generate_signal(&self, news: &NewsItem, sentiment: &SentimentResult) -> Option<Signal> {
    if sentiment.confidence < self.config.confidence_threshold {
        return None;
    }
    
    // 计算预期收益
    let expected_return = self.calc_expected_return(&news.market, sentiment.direction);
    
    if expected_return < self.config.min_expected_return {
        return None;
    }
    
    Some(Signal {
        market: news.market.clone(),
        direction: sentiment.direction,
        confidence: sentiment.confidence,
        expected_return,
    })
}

// 4. 合规检查
fn legal_check(&self, signal: &Signal) -> Result<()> {
    // 记录所有决策日志
    self.audit_log.log(signal);
    
    // 检查是否涉及内幕信息
    if self.config.legal_review_required {
        // 需要人工审查
        return Err(anyhow!("Legal review required"));
    }
    
    Ok(())
}
```

## ⚠️ 合规建议

1. **只使用公开数据源** - 避免任何非公开信息
2. **保留审计日志** - 记录所有决策依据
3. **咨询法律顾问** - 确保策略合法
4. **避免特定市场** - 某些司法管辖区的政治市场可能受限

## 📈 预期收益

```
马杜罗案例 (2024):
- 信息优势：提前知道选举结果
- 建仓价格：8¢
- 结算价格：$1.00
- 回报率：1200%+
- 获利：$400,000+

⚠️ 但这是内幕交易，已被起诉
```

**强烈建议仅使用公开另类数据，避免法律风险。**
