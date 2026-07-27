use anyhow::{Context, Result};
use common::{PaperEventKind, PaperRunReporter, RunMode, StrategyKind};
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

pub mod config;
pub mod copier;
pub mod monitor;
pub mod risk;

use crate::config::{Config, ExecutionMode};
use crate::copier::TradeCopier;
use crate::monitor::ChainMonitor;
use crate::risk::RiskManager;

pub struct Follower {
    config: Config,
    copier: TradeCopier,
    risk_manager: Arc<Mutex<RiskManager>>,
    running: bool,
}

impl Follower {
    pub fn new(config: Config) -> Result<Self> {
        let copier = TradeCopier::new(&config);

        Ok(Self {
            config,
            copier,
            risk_manager: Arc::new(Mutex::new(RiskManager::default())),
            running: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("Starting Follow Trader...");

        let mut reporter = PaperRunReporter::new(
            StrategyKind::FollowTrade,
            run_mode(self.config.execution.mode),
            "Follow Trade",
            None,
        );
        reporter.start(format!(
            "mode={:?} max_daily_loss={} max_position_per_market={}",
            self.config.execution.mode,
            self.config.strategy.max_daily_loss,
            self.config.strategy.max_position_per_market
        ));

        let (tx, mut rx) = mpsc::channel(100);

        let monitor_clone = ChainMonitor::new(&self.config);
        tokio::spawn(async move {
            if let Err(e) = monitor_clone.stream_trades(tx).await {
                warn!("Chain monitor stopped with error: {}", e);
            }
        });

        let mut trades_copied = 0;
        let mut total_realized_pnl = 0.0;

        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    info!("Ctrl-C received, shutting down gracefully...");
                    self.stop().await;
                    break;
                }
                trade_event = rx.recv() => {
                    match trade_event {
                        Some(trade) => {
                            reporter.update(
                                PaperEventKind::TradeObserved,
                                "Smart trade observed",
                                format!(
                                    "market={} side={:?} size_usd=${} price={}",
                                    trade.market_id, trade.side, trade.size_usd, trade.price
                                ),
                                None,
                                None,
                                |snapshot| {
                                    snapshot.metrics.trades_observed += 1;
                                },
                            );

                            let mut risk = self.risk_manager.lock().await;

                            if !risk.can_trade(&self.config) {
                                warn!("Risk limits exceeded, skipping trade from {}", trade.from);
                                reporter.warning(
                                    "Risk limits exceeded",
                                    format!("skip trade from={} market={}", trade.from, trade.market_id),
                                );
                                continue;
                            }

                            info!("Processing smart trade event: {:?}", trade);

                            match self.copier.copy(&trade).await {
                                Ok(outcome) => {
                                    let copied_notional = outcome.copied_notional_usd.to_f64().unwrap_or(0.0);
                                    risk.update_exposure(copied_notional);

                                    if let Some(realized_pnl) = outcome.realized_pnl {
                                        let pnl = realized_pnl.to_f64().unwrap_or(0.0);
                                        risk.update_pnl(pnl);
                                        total_realized_pnl += pnl;
                                    }

                                    info!(
                                        "Risk updated: market={} copied_notional=${:.2} shares={} total_position=${:.2} simulated={} order_id={:?}",
                                        trade.market_id,
                                        copied_notional,
                                        outcome.share_size,
                                        risk.total_position_value(),
                                        outcome.simulated,
                                        outcome.order_id
                                    );

                                    trades_copied += 1;
                                    info!(
                                        "Copied {} trades, realized PnL: ${}",
                                        trades_copied,
                                        total_realized_pnl
                                    );

                                    reporter.update(
                                        PaperEventKind::TradeCopied,
                                        if outcome.simulated {
                                            "Paper trade copied"
                                        } else {
                                            "Trade copied"
                                        },
                                        format!(
                                            "market={} copied_notional=${:.2} shares={} simulated={} total_position=${:.2}",
                                            trade.market_id,
                                            copied_notional,
                                            outcome.share_size,
                                            outcome.simulated,
                                            risk.total_position_value()
                                        ),
                                        None,
                                        outcome.realized_pnl.and_then(|value| value.to_f64()),
                                        |snapshot| {
                                            snapshot.metrics.trades_executed += 1;
                                            if outcome.simulated {
                                                snapshot.metrics.simulated_orders += 1;
                                            }
                                            snapshot.metrics.daily_pnl_usd = risk.daily_pnl;
                                            snapshot.metrics.total_pnl_usd = total_realized_pnl;
                                            snapshot.metrics.exposure_usd = risk.total_position_value();
                                        },
                                    );
                                }
                                Err(e) => {
                                    warn!("Failed to copy trade: {}", e);
                                    reporter.error(
                                        "Trade copy failed",
                                        format!("market={} error={}", trade.market_id, e),
                                    );
                                }
                            }
                        }
                        None => {
                            warn!("Trade event channel closed");
                            reporter.warning("Trade channel closed", "monitor channel closed unexpectedly");
                            break;
                        }
                    }
                }
            }
        }

        info!(
            "Follow Trader stopped. Copied: {} Realized PnL: ${}",
            trades_copied, total_realized_pnl
        );
        reporter.stop(format!(
            "copied_trades={} realized_pnl=${:.2}",
            trades_copied, total_realized_pnl
        ));
        Ok(())
    }

    pub async fn stop(&mut self) {
        self.running = false;
        let mut risk = self.risk_manager.lock().await;
        risk.reset_daily();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("follow_trade=info".parse()?),
        )
        .json()
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/follow-trade.toml".to_string());

    let config = Config::load(&config_path).context("Failed to load config")?;
    info!(
        "Config loaded: rpc_url={} mode={:?} environment={:?} live_ack={} fallback_to_paper={}",
        config.polygon.rpc_url,
        config.execution.mode,
        config.execution.environment,
        config.execution.live_acknowledged,
        config.execution.live_failure_fallback_to_paper
    );

    let mut follower = Follower::new(config)?;
    follower.run().await
}

fn run_mode(mode: ExecutionMode) -> RunMode {
    match mode {
        ExecutionMode::Paper => RunMode::Paper,
        ExecutionMode::Live => RunMode::Live,
    }
}
