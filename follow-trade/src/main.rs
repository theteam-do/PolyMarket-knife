//! Follow Trade - 跟单策略
//! 
//! 核心逻辑：监控聪明钱地址，自动复制交易

use anyhow::{Context, Result};
use tracing::{info, warn, error, instrument};

mod config;
mod monitor;
mod copier;
mod risk;

use config::Config;
use monitor::ChainMonitor;
use copier::TradeCopier;
use risk::RiskManager;

pub struct Follower {
    config: Config,
    monitor: ChainMonitor,
    copier: TradeCopier,
    risk_manager: RiskManager,
    running: bool,
}

impl Follower {
    pub fn new(config: Config) -> Self {
        Self {
            monitor: ChainMonitor::new(&config),
            copier: TradeCopier::new(&config),
            risk_manager: RiskManager::new(&config.strategy),
            config,
            running: false,
        }
    }

    #[instrument(skip(self), fields(name = "follow_trade"))]
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("Follow Trader starting...");
        info!("Monitoring {} smart addresses", self.config.strategy.smart_addresses.len());

        // 持续监听链上事件
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_millis(100)
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
        // 1. 获取聪明钱交易事件
        let trades = self.monitor.fetch_trades().await?;

        for trade in trades {
            info!("💰 Smart money trade detected: {:?}", trade);

            // 2. 风控检查
            if !self.risk_manager.can_trade(&trade) {
                warn!("⛔ Risk check failed, skipping trade");
                continue;
            }

            // 3. 执行跟单
            match self.copier.copy(&trade).await {
                Ok(_) => {
                    info!("✅ Trade copied successfully");
                }
                Err(e) => {
                    warn!("❌ Failed to copy trade: {}", e);
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
                .add_directive("follow_trade=info".parse()?)
        )
        .json()
        .init();

    // 加载配置
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/follow-trade.toml".to_string());
    
    let config = Config::load(&config_path)
        .context("Failed to load config")?;

    let mut follower = Follower::new(config);
    
    // 处理信号
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutting down...");
    });

    follower.run().await
}
