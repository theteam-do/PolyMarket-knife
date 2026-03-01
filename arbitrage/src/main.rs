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
use poly_client::ws_client::{WsClient, WsConfig, ChannelType, WsMessage};
use tokio::sync::mpsc;
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

    let ws_market_url = config.clob.ws_market_url.clone().unwrap_or_else(|| "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string());
    let ws_user_url = config.clob.ws_user_url.clone().unwrap_or_else(|| "wss://ws-subscriptions-clob.polymarket.com/ws/user".to_string());

    let ws_config = WsConfig {
        market_url: ws_market_url,
        user_url: ws_user_url,
        ..Default::default()
    };
    let ws_client = WsClient::new(ws_config);

    let scanner = Scanner::new(&config);
    let detector = Arc::new(Detector::new(&config.strategy));
    let executor = Arc::new(Executor::new(&config));

    info!("Fetching initial market state via HTTP...");
    let initial_markets = scanner.scan().await?;
    let mut state = MarketState::new(initial_markets);
    
    let asset_ids = state.get_all_assets();
    info!("Initial market state loaded. Tracking {} assets. Subscribing to Market WS...", asset_ids.len());

    let (tx, mut rx) = mpsc::channel(1000);
    
    let ws_client = Arc::new(ws_client);
    let ws_client_clone = Arc::clone(&ws_client);
    
    tokio::spawn(async move {
        if let Err(e) = ws_client_clone.stream_with_reconnect(ChannelType::Market, asset_ids, tx).await {
            warn!("WebSocket stream error: {}", e);
        }
    });

    info!("Arbitrage initialized and waiting for real-time WS events...");

    while let Some(msg) = rx.recv().await {
        match msg {
            WsMessage::MarketEvent { event_type, payload } => {
                if state.update_from_ws_payload(&event_type, &payload) {
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
            },
            _ => {} // Ignore generic or raw messages for now
        }
    }

    Ok(())
}
