//! Arbitrage - 简化工作版本

use anyhow::{Context, Result};
use tracing::info;

mod config;
mod scanner;
mod detector;
mod executor;

use config::Config;
use scanner::Scanner;
use detector::Detector;
use executor::Executor;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("arbitrage=info".parse()?)
        )
        .init();

    info!("Arbitrage starting...");

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/arbitrage.toml".to_string());
    
    let config = Config::load(&config_path)
        .context("Failed to load config")?;

    let _scanner = Scanner::new(&config);
    let _detector = Detector::new(&config.strategy);
    let _executor = Executor::new(&config);
    
    info!("Arbitrage initialized");

    Ok(())
}
