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
mod executor;
mod nlp;
mod signal;

use collector::NewsCollector;
use compliance::ComplianceChecker;
use config::Config;
use executor::Executor;
use nlp::NLPEngine;
use signal::SignalGenerator;

pub struct InfoTrader {
    #[allow(dead_code)]
    config: Config,
    collector: NewsCollector,
    nlp_engine: NLPEngine,
    signal_gen: SignalGenerator,
    compliance: ComplianceChecker,
    executor: Option<Executor>,
    running: bool,
}

impl InfoTrader {
    pub async fn new(config: Config) -> Result<Self> {
        let executor = match Executor::new(&config).await {
            Ok(executor) => Some(executor),
            Err(err) => {
                warn!("Trading executor disabled: {}", err);
                None
            }
        };

        Ok(Self {
            collector: NewsCollector::new(&config.sources, config.clob.proxy_url.as_deref()),
            nlp_engine: NLPEngine::new(&config.sources),
            signal_gen: SignalGenerator::new(&config.strategy),
            compliance: ComplianceChecker::new(&config.risk),
            executor,
            config,
            running: false,
        })
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
        let news = self.collector.fetch().await?;

        for item in news {
            info!(
                "Processing news from source={} ts={} has_market={} text_len={}",
                item.source,
                item.timestamp,
                item.market.is_some(),
                item.content.len()
            );
            let sentiment = self.nlp_engine.analyze(&item);
            info!(
                "Sentiment analyzed: direction={:?} confidence={:.3} matched_keywords={:?}",
                sentiment.direction, sentiment.confidence, sentiment.matched_keywords
            );

            if let Some(signal) = self.signal_gen.generate(&item, &sentiment) {
                info!(
                    "🎯 Signal generated: {:?}, expected_return={:.3}",
                    signal, signal.expected_return
                );

                if let Err(e) = self.compliance.check(&signal) {
                    warn!("⛔ Compliance check failed: {}", e);
                    continue;
                }

                match self.execute(&signal).await {
                    Ok(_) => {
                        self.compliance.update_pnl(Decimal::ZERO);
                        info!("✅ Trade submitted");
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
        let executor = self
            .executor
            .as_ref()
            .context("trading executor unavailable; check CLOB config and private key")?;
        executor.execute(signal).await
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.compliance.reset_daily();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info_edge=info".parse()?),
        )
        .json()
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/info-edge.toml".to_string());

    let config = Config::load(&config_path).context("Failed to load config")?;
    let mut trader = InfoTrader::new(config).await?;

    trader.run().await
}
