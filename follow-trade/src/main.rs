use anyhow::{Context, Result};
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

pub mod config;
pub mod copier;
pub mod monitor;

use crate::config::Config;
use crate::copier::TradeCopier;
use crate::monitor::ChainMonitor;

#[derive(Default)]
struct RiskManager {
    daily_pnl: f64,
    total_exposure: f64,
}

impl RiskManager {
    fn can_trade(&self, config: &Config) -> bool {
        self.daily_pnl >= -config.strategy.max_daily_loss
            && self.total_exposure <= config.strategy.max_position_per_market
    }

    fn update_exposure(&mut self, notional: f64) {
        self.total_exposure += notional;
    }

    fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
    }

    fn total_position_value(&self) -> f64 {
        self.total_exposure
    }

    fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
    }
}

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

        let (tx, mut rx) = mpsc::channel(100);

        // 启动独立任务监控链上日志
        let monitor_clone = ChainMonitor::new(&self.config);
        tokio::spawn(async move {
            if let Err(e) = monitor_clone.stream_trades(tx).await {
                warn!("Chain monitor stopped with error: {}", e);
            }
        });

        let mut trades_copied = 0;
        let mut total_pnl = 0.0;

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
                            let mut risk = self.risk_manager.lock().await;
                            
                            if !risk.can_trade(&self.config) {
                                warn!("Risk limits exceeded, skipping trade from {}", trade.from);
                                continue;
                            }

                            info!("Processing smart trade event: {:?}", trade);

                            match self.copier.copy(&trade).await {
                                Ok(profit_dec) => {
                                    let profit = profit_dec.to_f64().unwrap_or(0.0);
                                    let size_f64 = trade.size_usd.to_f64().unwrap_or(0.0);
                                    let copied_notional = size_f64 * self.config.strategy.copy_ratio;
                                    
                                    risk.update_exposure(copied_notional);
                                    risk.update_pnl(profit);
                                    
                                    info!(
                                        "Risk updated: market={} copied_notional=${:.2} total_position=${:.2}",
                                        trade.market_id,
                                        copied_notional,
                                        risk.total_position_value()
                                    );
                                    
                                    trades_copied += 1;
                                    total_pnl += profit;
                                    info!("Copied {} trades, total PnL: ${}", trades_copied, total_pnl);
                                }
                                Err(e) => {
                                    warn!("Failed to copy trade: {}", e);
                                }
                            }
                        }
                        None => {
                            warn!("Trade event channel closed");
                            break;
                        }
                    }
                }
            }
        }

        info!(
            "Follow Trader stopped. Copied: {} PnL: ${}",
            trades_copied, total_pnl
        );
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
