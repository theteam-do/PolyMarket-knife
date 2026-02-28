//! Arbitrage - 简化工作版本

use anyhow::{Context, Result};
use tracing::info;

mod config;
mod detector;
mod executor;
mod scanner;

use config::Config;
use detector::Detector;
use executor::Executor;
use scanner::Scanner;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("arbitrage=info".parse()?),
        )
        .init();

    info!("Arbitrage starting...");

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/arbitrage.toml".to_string());

    let config = Config::load(&config_path).context("Failed to load config")?;
    info!(
        "Config loaded: rpc_url={}, gas_price_gwei={}",
        config.polygon.rpc_url, config.strategy.gas_price_gwei
    );

    let scanner = Scanner::new(&config);
    let detector = Detector::new(&config.strategy);
    let executor = Executor::new(&config);

    let markets = scanner.scan().await?;
    if let Some(opportunity) = detector.detect(&markets) {
        let expected_profit = executor.execute(&opportunity).await?;
        info!("Opportunity executed: {} expected_profit={}", opportunity, expected_profit);
    } else {
        info!("No arbitrage opportunity found");
    }

    sleep(Duration::from_millis(detector.scan_interval_ms())).await;

    info!("Arbitrage initialized");

    Ok(())
}
