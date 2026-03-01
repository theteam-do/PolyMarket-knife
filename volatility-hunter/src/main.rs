//! Volatility Hunter - 波动狩猎完整实现

use anyhow::{Context, Result};
use tracing::{info, warn};

mod binance_ws;
mod config;
mod executor;
mod risk;
mod signal;

use binance_ws::BinanceFeed;
use config::Config;
use executor::Executor;
use risk::RiskManager;
use signal::SignalGenerator;

pub struct Hunter {
    config: Config,
    signal_gen: SignalGenerator,
    executor: Executor,
    risk_manager: RiskManager,
    running: bool,
}

impl Hunter {
    pub fn new(config: Config) -> Result<Self> {
        let signal_gen = SignalGenerator::new(&config.strategy);
        let executor = Executor::new(&config);
        let risk_manager = RiskManager::new(&config.strategy);

        Ok(Self {
            config,
            signal_gen,
            executor,
            risk_manager,
            running: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("Volatility Hunter starting...");
        info!("Monitoring symbols: {:?}", self.config.strategy.symbols);

        // 启动币安数据流
        let (tx, mut rx) = tokio::sync::mpsc::channel(1000);
        let binance_feed = BinanceFeed::new(&self.config.binance);

        let symbols = self.config.strategy.symbols.clone();
        tokio::spawn(async move {
            if let Err(e) = binance_feed.stream(tx, symbols).await {
                warn!("Binance feed error: {}", e);
            }
        });

        let mut signals_generated = 0u32;
        let mut trades_executed = 0u32;
        let mut total_pnl = rust_decimal::Decimal::ZERO;

        while self.running {
            if let Some(tick) = rx.recv().await {
                match self.on_tick(tick).await {
                    Ok(Some(profit)) => {
                        signals_generated += 1;
                        trades_executed += 1;
                        total_pnl += profit;
                        info!(
                            "Signals: {} Trades: {} PnL: ${}",
                            signals_generated, trades_executed, total_pnl
                        );
                    }
                    Ok(None) => {}
                    Err(e) => {
                        warn!("Tick error: {}", e);
                    }
                }
            }
        }

        info!(
            "Hunter stopped. Signals: {} Trades: {} PnL: ${}",
            signals_generated, trades_executed, total_pnl
        );
        Ok(())
    }

    async fn on_tick(&mut self, tick: PriceTick) -> Result<Option<rust_decimal::Decimal>> {
        // 1. 生成信号
        if let Some(signal) = self.signal_gen.generate(&tick) {
            info!("Signal generated: {:?}", signal);

            // 2. 风控检查
            if !self.risk_manager.can_trade(&signal) {
                warn!("Risk check failed");
                return Ok(None);
            }

            // 3. 执行交易
            match self.executor.execute(&signal).await {
                Ok(profit) => {
                    self.risk_manager.update_pnl(profit);
                    return Ok(Some(profit));
                }
                Err(e) => {
                    warn!("Execution failed: {}", e);
                    self.risk_manager.update_pnl(rust_decimal::Decimal::ZERO);
                }
            }
        }

        Ok(None)
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.risk_manager.reset_daily();
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("volatility_hunter=info".parse()?),
        )
        .json()
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/volatility-hunter.toml".to_string());

    let config = Config::load(&config_path).context("Failed to load config")?;
    info!(
        "Config loaded: rpc_url={} mode={:?} environment={:?} live_ack={} fallback_to_paper={}",
        config.polygon.rpc_url,
        config.execution.mode,
        config.execution.environment,
        config.execution.live_acknowledged,
        config.execution.live_failure_fallback_to_paper
    );

    let mut hunter = Hunter::new(config)?;

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutting down...");
    });

    hunter.run().await
}
