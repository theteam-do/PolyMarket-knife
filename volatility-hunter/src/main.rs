//! Volatility Hunter - 超短期波动狩猎
//! 
//! 核心逻辑：利用币安数据比 Polymarket 快的优势，捕捉波动瞬间的定价延迟
//! 延迟目标：<20ms 端到端

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error, instrument};

mod config;
mod binance_ws;
mod signal;
mod executor;
mod risk;

use config::Config;
use binance_ws::BinanceFeed;
use signal::SignalGenerator;
use executor::Executor;
use risk::RiskManager;

pub struct Hunter {
    config: Config,
    signal_gen: SignalGenerator,
    executor: Executor,
    risk_manager: Arc<RiskManager>,
    running: bool,
}

impl Hunter {
    pub fn new(config: Config) -> Self {
        Self {
            signal_gen: SignalGenerator::new(&config.strategy),
            executor: Executor::new(&config),
            risk_manager: Arc::new(RiskManager::new(&config.strategy)),
            config,
            running: false,
        }
    }

    #[instrument(skip(self), fields(name = "volatility_hunter"))]
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("🚀 Volatility Hunter starting...");
        info!("Monitoring symbols: {:?}", self.config.strategy.symbols);

        // 启动币安数据流
        let (tx, mut rx) = mpsc::channel(1000);
        let binance_feed = BinanceFeed::new(&self.config.binance);
        
        let symbols = self.config.strategy.symbols.clone();
        tokio::spawn(async move {
            if let Err(e) = binance_feed.stream(tx, symbols).await {
                error!("Binance feed error: {}", e);
            }
        });

        // 主事件循环
        let mut processed = 0u64;
        while self.running {
            if let Some(tick) = rx.recv().await {
                if let Err(e) = self.on_tick(tick).await {
                    error!("Tick error: {}", e);
                }
                
                processed += 1;
                if processed % 1000 == 0 {
                    info!("📊 Processed {} ticks", processed);
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self), fields(symbol = %tick.symbol, price = tick.price))]
    async fn on_tick(&mut self, tick: PriceTick) -> Result<()> {
        // 1. 生成交易信号
        if let Some(signal) = self.signal_gen.generate(&tick) {
            info!("🎯 Signal generated: {:?}", signal);

            // 2. 风控检查
            if !self.risk_manager.can_trade(&signal) {
                warn!("⛔ Risk check failed");
                return Ok(());
            }

            // 3. 执行交易
            match self.executor.execute(&signal).await {
                Ok(_) => {
                    info!("✅ Trade executed");
                }
                Err(e) => {
                    warn!("❌ Execution failed: {}", e);
                }
            }
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub symbol: String,
    pub price: f64,
    pub timestamp: u64,
    pub volume: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("volatility_hunter=info".parse()?)
        )
        .json()
        .init();

    // 加载配置
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/volatility-hunter.toml".to_string());
    
    let config = Config::load(&config_path)
        .context("Failed to load config")?;

    let mut hunter = Hunter::new(config);
    
    // 处理信号
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutting down...");
    });

    hunter.run().await
}
