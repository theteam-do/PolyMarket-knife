//! Market Maker - 简化版本

use anyhow::{Context, Result};
use tracing::info;

mod config;
mod executor;

use config::Config;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("market_maker=info".parse()?)
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/market-maker.toml".to_string());
    
    let config = Config::load(&config_path)
        .context("Failed to load config")?;

    let _executor = Executor::new(&config);
    
    info!("Market Maker started");

    Ok(())
}
use executor::Executor;
