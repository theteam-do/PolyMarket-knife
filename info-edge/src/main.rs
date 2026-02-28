//! Info Edge - 信息差交易策略
//! 
//! ⚠️ 警告：本策略可能涉及法律风险，仅供学习研究
//! 
//! 核心逻辑：NLP 监控新闻源，比市场更早知道重大事件

use anyhow::{Context, Result};
use tracing::{info, warn, error, instrument};

mod config;
mod collector;
mod nlp;
mod signal;
mod compliance;

use config::Config;
use collector::NewsCollector;
use nlp::NLPEngine;
use signal::SignalGenerator;
use compliance::ComplianceChecker;

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

        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(1)
        );

        while self.running {
            interval.tick().await;
            
            if let Err(e) = self.tick().await {
                error!("Tick error: {}", e);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn tick(&mut self) -> Result<()> {
        // 1. 抓取新闻
        let news = self.collector.fetch().await?;

        for item in news {
            // 2. NLP 分析
            let sentiment = self.nlp_engine.analyze(&item);

            // 3. 生成信号
            if let Some(signal) = self.signal_gen.generate(&item, &sentiment) {
                info!("🎯 Signal generated: {:?}", signal);

                // 4. 合规检查 ⚠️
                if let Err(e) = self.compliance.check(&signal) {
                    warn!("⛔ Compliance check failed: {}", e);
                    continue;
                }

                // 5. 执行交易
                match self.execute(&signal).await {
                    Ok(_) => {
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

    async fn execute(&self, _signal: &signal::Signal) -> Result<()> {
        // TODO: 在 Polymarket CLOB 下单
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info_edge=info".parse()?)
        )
        .json()
        .init();

    // 加载配置
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/info-edge.toml".to_string());
    
    let config = Config::load(&config_path)
        .context("Failed to load config")?;

    let mut trader = InfoTrader::new(config);
    
    // 处理信号
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutting down...");
    });

    trader.run().await
}
