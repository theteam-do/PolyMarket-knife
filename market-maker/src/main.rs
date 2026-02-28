//! Market Maker - 简化工作版本

use anyhow::{Context, Result};
use tracing::info;

mod config;
mod executor;
mod order_book;
mod quoting;
mod risk;

use config::Config;
use executor::Executor;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("market_maker=info".parse()?)
        )
        .init();

    info!("Market Maker starting up...");

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/market-maker.toml".to_string());
    
    let config = Config::load(&config_path)
        .context("Failed to load config")?;

    let _executor = Executor::new(&config);
    
    info!("Market Maker initialized");

    Ok(())
}
