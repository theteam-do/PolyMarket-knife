//! Arbitrage - 简化工作版本

use anyhow::{Context, Result};
use tracing::{info, warn};

mod config;
mod detector;
mod executor;
mod scanner;
mod state;

use config::Config;
use detector::Detector;
use executor::Executor;
use scanner::Scanner;
use state::MarketState;
use polymarket_client_sdk::clob::ws::Client as WsClient;
use futures::StreamExt;
use std::sync::Arc;

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
        "Config loaded: rpc_url={} gas_price_gwei={} mode={:?} environment={:?} live_ack={}",
        config.polygon.rpc_url,
        config.strategy.gas_price_gwei,
        config.execution.mode,
        config.execution.environment,
        config.execution.live_acknowledged
    );

    let scanner = Scanner::new(&config);
    let detector = Arc::new(Detector::new(&config.strategy));
    let executor = Arc::new(Executor::new(&config));

    info!("Fetching initial market state via HTTP...");
    let initial_markets = scanner.scan().await?;
    let mut state = MarketState::new(initial_markets);
    
    let asset_ids = state.get_all_assets();
    info!("Initial market state loaded. Tracking {} assets. Subscribing to Market WS...", asset_ids.len());

    // 使用官方 SDK 的 WebSocket 客户端
    let ws_client = WsClient::default();
    let stream = ws_client.subscribe_orderbook(asset_ids)
        .context("Failed to subscribe to orderbook")?;

    info!("Arbitrage initialized and waiting for real-time WS events...");

    let mut stream = Box::pin(stream);
    while let Some(book_result) = stream.next().await {
        match book_result {
            Ok(book) => {
                // 更新市场状态
                if state.update_from_orderbook(&book) {
                    let markets = state.get_all_markets();
                    if let Some(opportunity) = detector.detect(&markets) {
                        info!("Opportunity detected via WS: {}", opportunity);
                        match executor.execute(&opportunity).await {
                            Ok(expected_profit) => {
                                info!("Opportunity executed: expected_profit={}", expected_profit);
                            }
                            Err(e) => {
                                warn!("Failed to execute opportunity: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("WebSocket error: {}", e);
            }
        }
    }

    Ok(())
}
