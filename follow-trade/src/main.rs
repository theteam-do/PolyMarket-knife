//! Follow Trade - 跟单策略完整实现

use anyhow::{Context, Result};
use tracing::{info, warn};

mod config;
mod copier;
mod monitor;
mod risk;

use config::Config;
use copier::TradeCopier;
use monitor::ChainMonitor;
use risk::RiskManager;

pub struct Follower {
    config: Config,
    monitor: ChainMonitor,
    copier: TradeCopier,
    risk_manager: RiskManager,
    running: bool,
}

impl Follower {
    pub fn new(config: Config) -> Result<Self> {
        let monitor = ChainMonitor::new(&config);
        let copier = TradeCopier::new(&config);
        let risk_manager = RiskManager::new(&config.strategy);

        Ok(Self {
            config,
            monitor,
            copier,
            risk_manager,
            running: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("Follow Trader starting...");
        info!(
            "Monitoring {} smart addresses",
            self.config.strategy.smart_addresses.len()
        );

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

        let mut trades_copied = 0u32;
        let mut total_pnl = rust_decimal::Decimal::ZERO;

        while self.running {
            interval.tick().await;

            match self.tick().await {
                Ok(Some(profit)) => {
                    trades_copied += 1;
                    total_pnl += profit;
                    info!("Copied {} trades, total PnL: ${}", trades_copied, total_pnl);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Tick error: {}", e);
                }
            }
        }

        info!(
            "Follow Trader stopped. Copied: {} PnL: ${}",
            trades_copied, total_pnl
        );
        Ok(())
    }

    async fn tick(&mut self) -> Result<Option<rust_decimal::Decimal>> {
        // 1. 获取聪明钱交易
        let trades = self.monitor.fetch_trades().await?;

        for trade in trades {
            // 2. 风控检查
            if !self.risk_manager.can_trade(&trade) {
                warn!("Risk check failed for trade");
                continue;
            }

            // 3. 执行跟单
            match self.copier.copy(&trade).await {
                Ok(profit) => {
                    let copied_notional = trade.size_usd * self.config.strategy.copy_ratio;
                    self.risk_manager
                        .update_position(&trade.market_id, copied_notional);
                    self.risk_manager
                        .update_pnl(profit.to_string().parse::<f64>().unwrap_or(0.0));
                    info!(
                        "Risk updated: market={} copied_notional=${:.2} total_position=${:.2}",
                        trade.market_id,
                        copied_notional,
                        self.risk_manager.total_position_value()
                    );
                    return Ok(Some(profit));
                }
                Err(e) => {
                    warn!("Failed to copy trade: {}", e);
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

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutting down...");
    });

    follower.run().await
}
