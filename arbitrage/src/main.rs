//! Arbitrage - 套利策略
//! 
//! 核心逻辑：捕捉 Yes + No ≠ $1 的定价错误

use anyhow::{Context, Result};
use tracing::{info, warn, error, instrument};

mod config;
mod scanner;
mod detector;
mod executor;

use config::Config;
use scanner::Scanner;
use detector::Detector;
use executor::Executor;

pub struct Arbitrageur {
    config: Config,
    scanner: Scanner,
    detector: Detector,
    executor: Executor,
    running: bool,
}

impl Arbitrageur {
    pub fn new(config: Config) -> Self {
        Self {
            scanner: Scanner::new(&config),
            detector: Detector::new(&config.strategy),
            executor: Executor::new(&config),
            config,
            running: false,
        }
    }

    #[instrument(skip(self), fields(name = "arbitrage"))]
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("Arbitrageur starting...");

        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_millis(self.config.strategy.scan_interval_ms)
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
        // 1. 扫描所有市场价格
        let prices = self.scanner.scan().await?;

        if prices.is_empty() {
            return Ok(());
        }

        // 2. 检测套利机会
        if let Some(opp) = self.detector.detect(&prices) {
            info!("🎯 Arbitrage opportunity detected: {:?}", opp);

            // 3. 执行套利
            match self.executor.execute(&opp).await {
                Ok(profit) => {
                    info!("✅ Arbitrage executed, estimated profit: ${:.4}", profit);
                }
                Err(e) => {
                    warn!("❌ Arbitrage execution failed: {}", e);
                }
            }
        }

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
                .add_directive("arbitrage=info".parse()?)
        )
        .json()
        .init();

    // 加载配置
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/arbitrage.toml".to_string());
    
    let config = Config::load(&config_path)
        .context("Failed to load config")?;

    let mut arb = Arbitrageur::new(config);
    
    // 处理信号
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutting down...");
    });

    arb.run().await
}
