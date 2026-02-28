//! Info Edge - 信息差交易策略
//!
//! ⚠️ 警告：本策略可能涉及法律风险，仅供学习研究
//!
//! 核心逻辑：NLP 监控新闻源，比市场更早知道重大事件

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use tracing::{error, info, instrument, warn};

mod collector;
mod compliance;
mod config;
mod nlp;
mod signal;

use collector::NewsCollector;
use compliance::ComplianceChecker;
use config::Config;
use nlp::NLPEngine;
use signal::SignalGenerator;

pub struct InfoTrader {
    config: Config,
    collector: NewsCollector,
    nlp_engine: NLPEngine,
    signal_gen: SignalGenerator,
    compliance: ComplianceChecker,
    running: bool,
}

impl InfoTrader {
    pub fn new(config: Config) -> Self {
        Self {
            collector: NewsCollector::new(&config.sources),
            nlp_engine: NLPEngine::new(&config.sources),
            signal_gen: SignalGenerator::new(&config.strategy),
            compliance: ComplianceChecker::new(&config.risk),
            config,
            running: false,
        }
    }

    #[instrument(skip(self), fields(name = "info_edge"))]
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("📰 Info Trader starting...");
        info!("Monitoring {} news sources", self.collector.source_count());

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        while self.running {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting down...");
                    self.stop();
                }
                _ = interval.tick() => {
                    if let Err(e) = self.tick().await {
                        error!("Tick error: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn tick(&mut self) -> Result<()> {
        // 1. 抓取新闻
        let news = self.collector.fetch().await?;

        for item in news {
            info!("Processing news from source={}", item.source);
            // 2. NLP 分析
            let sentiment = self.nlp_engine.analyze(&item);
            info!(
                "Sentiment analyzed: direction={:?} confidence={:.3} matched_keywords={:?}",
                sentiment.direction,
                sentiment.confidence,
                sentiment.matched_keywords
            );

            // 3. 生成信号
            if let Some(signal) = self.signal_gen.generate(&item, &sentiment) {
                info!(
                    "🎯 Signal generated: {:?}, expected_return={:.3}",
                    signal, signal.expected_return
                );

                // 4. 合规检查 ⚠️
                if let Err(e) = self.compliance.check(&signal) {
                    warn!("⛔ Compliance check failed: {}", e);
                    continue;
                }

                // 5. 执行交易
                match self.execute(&signal).await {
                    Ok(_) => {
                        self.compliance.update_pnl(Decimal::ZERO);
                        info!("✅ Trade executed");
                    }
                    Err(e) => {
                        warn!("❌ Execution failed: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn execute(&self, signal: &signal::Signal) -> Result<()> {
        let endpoint = format!("{}/order", self.config.clob.host.trim_end_matches('/'));
        let side = match signal.direction {
            nlp::Direction::Yes => "BUY",
            nlp::Direction::No => "SELL",
            nlp::Direction::Neutral => return Ok(()),
        };

        let size = (self.config.strategy.max_position_usd * signal.confidence.max(0.1)).max(1.0);
        let payload = serde_json::json!({
            "market": signal.market,
            "side": side,
            "size_usd": size,
            "confidence": signal.confidence,
            "expected_return": signal.expected_return,
            "news_title": signal.news_title,
        });

        let client = reqwest::Client::new();
        let response = client.post(endpoint).json(&payload).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Execution rejected {}: {}", status, body);
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.compliance.reset_daily();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info_edge=info".parse()?),
        )
        .json()
        .init();

    // 加载配置
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/info-edge.toml".to_string());

    let config = Config::load(&config_path).context("Failed to load config")?;

    let mut trader = InfoTrader::new(config);

    trader.run().await
}
