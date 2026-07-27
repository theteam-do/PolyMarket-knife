//! Volatility Hunter - 波动狩猎完整实现

use anyhow::{Context, Result};
use common::{PaperEventKind, PaperRunReporter, RunMode, StrategyKind};
use rust_decimal::prelude::ToPrimitive;
use tracing::{info, warn};

mod binance_ws;
mod config;
mod executor;
mod risk;
mod signal;

use binance_ws::BinanceFeed;
use config::{Config, ExecutionMode};
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

        let mut reporter = PaperRunReporter::new(
            StrategyKind::VolatilityHunter,
            run_mode(self.config.execution.mode),
            "Volatility Hunter",
            None,
        );
        reporter.start(format!(
            "mode={:?} symbols={} volatility_threshold={} momentum_threshold={}",
            self.config.execution.mode,
            self.config.strategy.symbols.len(),
            self.config.strategy.volatility_threshold,
            self.config.strategy.momentum_threshold
        ));

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
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutdown signal received");
                    self.stop();
                }
                tick = rx.recv() => {
                    match tick {
                        Some(tick) => {
                            match self.on_tick(tick, &mut reporter).await {
                                Ok((signal_generated, profit)) => {
                                    if signal_generated {
                                        signals_generated += 1;
                                    }
                                    if let Some(profit) = profit {
                                        trades_executed += 1;
                                        total_pnl += profit;
                                        info!(
                                            "Signals: {} Trades: {} PnL: ${}",
                                            signals_generated, trades_executed, total_pnl
                                        );
                                    }
                                }
                                Err(e) => {
                                    warn!("Tick error: {}", e);
                                    reporter.error("Tick error", e.to_string());
                                }
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        info!(
            "Hunter stopped. Signals: {} Trades: {} PnL: ${}",
            signals_generated, trades_executed, total_pnl
        );
        reporter.stop(format!(
            "signals={} trades={} pnl=${}",
            signals_generated, trades_executed, total_pnl
        ));
        Ok(())
    }

    async fn on_tick(
        &mut self,
        tick: PriceTick,
        reporter: &mut PaperRunReporter,
    ) -> Result<(bool, Option<rust_decimal::Decimal>)> {
        // 1. 生成信号
        if let Some(signal) = self.signal_gen.generate(&tick) {
            info!("Signal generated: {:?}", signal);
            let estimated_position = self.executor.estimate_position_usd(&signal);
            reporter.update(
                PaperEventKind::SignalGenerated,
                "Signal generated",
                format!(
                    "symbol={} confidence={:.2} estimated_position=${:.2}",
                    signal.symbol(),
                    signal.confidence(),
                    estimated_position
                ),
                None,
                None,
                |snapshot| {
                    snapshot.metrics.signals_generated += 1;
                    snapshot.metrics.exposure_usd = estimated_position.to_f64().unwrap_or(0.0);
                },
            );

            // 2. 风控检查
            if !self.risk_manager.can_trade(&signal) {
                warn!("Risk check failed");
                reporter.warning(
                    "Risk check failed",
                    format!(
                        "symbol={} confidence={:.2}",
                        signal.symbol(),
                        signal.confidence()
                    ),
                );
                return Ok((true, None));
            }

            // 3. 执行交易
            match self.executor.execute(&signal).await {
                Ok(profit) => {
                    self.risk_manager.update_pnl(profit);
                    let profit_f64 = profit.to_f64().unwrap_or(0.0);
                    reporter.update(
                        PaperEventKind::ExecutionSimulated,
                        if self.config.execution.mode == ExecutionMode::Paper {
                            "Paper execution simulated"
                        } else {
                            "Execution completed"
                        },
                        format!(
                            "symbol={} profit=${:.2} estimated_position=${:.2}",
                            signal.symbol(),
                            profit_f64,
                            estimated_position
                        ),
                        None,
                        Some(profit_f64),
                        |snapshot| {
                            snapshot.metrics.trades_executed += 1;
                            if self.config.execution.mode == ExecutionMode::Paper {
                                snapshot.metrics.simulated_orders += 1;
                            }
                            snapshot.metrics.daily_pnl_usd += profit_f64;
                            snapshot.metrics.total_pnl_usd += profit_f64;
                            snapshot.metrics.exposure_usd =
                                estimated_position.to_f64().unwrap_or(0.0);
                        },
                    );
                    return Ok((true, Some(profit)));
                }
                Err(e) => {
                    warn!("Execution failed: {}", e);
                    self.risk_manager.update_pnl(rust_decimal::Decimal::ZERO);
                    reporter.error(
                        "Execution failed",
                        format!("symbol={} error={}", signal.symbol(), e),
                    );
                }
            }

            return Ok((true, None));
        }

        Ok((false, None))
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

    hunter.run().await
}

fn run_mode(mode: ExecutionMode) -> RunMode {
    match mode {
        ExecutionMode::Paper => RunMode::Paper,
        ExecutionMode::Live => RunMode::Live,
    }
}
