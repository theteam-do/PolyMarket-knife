use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub polygon: PolygonConfig,
    pub clob: ClobConfig,
    pub sources: SourcesConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PolygonConfig {
    pub rpc_url: String,
    #[serde(skip)]
    pub private_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClobConfig {
    pub host: String,
    pub ws_market_url: Option<String>,
    pub ws_user_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourcesConfig {
    /// 新闻源 API 列表
    pub news_apis: Vec<NewsApiConfig>,

    /// 监控关键词
    pub keywords: Vec<String>,

    /// 政府网站监控列表
    pub gov_websites: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewsApiConfig {
    pub name: String,
    pub url: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    /// 置信度阈值
    pub confidence_threshold: f64,

    /// 最大仓位 (美元)
    pub max_position_usd: f64,

    /// 最小预期收益 (0.3 = 30%)
    pub min_expected_return: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RiskConfig {
    /// 日最大亏损 (美元)
    pub max_daily_loss: f64,

    /// 是否需要法律审查 ⚠️
    pub legal_review_required: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
