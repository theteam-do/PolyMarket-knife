//! Arbitrage - 简化工作版本

use anyhow::{Context, Result};
use common::{PaperEventKind, PaperRunReporter, RunMode, StrategyKind};
use tracing::{error, info, warn};

mod config;
mod detector;
mod executor;
mod probability;
mod processor;
mod quant;
mod replay;
mod reporting;
mod scanner;
mod settlement;
mod state;

use config::{Config, ExecutionMode};
use detector::Detector;
use executor::Executor;
use futures::StreamExt;
use polymarket_client_sdk::clob::ws::Client as WsClient;
use polymarket_client_sdk::ws::config::Config as WsConfig;
use processor::process_book_update;
use scanner::Scanner;
use state::MarketState;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                "arbitrage=info"
                    .parse()
                    .expect("Failed to parse log directive"),
            ),
        )
        .init();

    info!("Arbitrage starting...");

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/arbitrage.toml".to_string());

    let config = Config::load(&config_path).context("Failed to load config")?;
    info!(
        "Config loaded: rpc_url={} gas_price_gwei={} mode={:?} environment={:?} live_ack={} ctf_enabled={}",
        config.polygon.rpc_url,
        config.strategy.gas_price_gwei,
        config.execution.mode,
        config.execution.environment,
        config.execution.live_acknowledged,
        config.ctf.enabled
    );
    info!(
        "Quant config: fees_bps={} slippage_bps={} posterior_override={:?} probability_enabled={} probability_prior_prob={} apply_kelly_sizing={} bankroll_usd={:?} max_notional_usd={:?}",
        config.quant.fees_bps,
        config.quant.slippage_bps,
        config.quant.posterior_prob_override,
        config.quant.probability.enabled,
        config.quant.probability.prior_prob,
        config.quant.apply_kelly_sizing,
        config.quant.bankroll_usd,
        config.quant.max_notional_usd
    );

    let scanner = Scanner::new(&config);
    let detector = Arc::new(Detector::new(&config.strategy, &config.quant));
    let executor = Arc::new(Executor::new(&config));
    let mut reporter = PaperRunReporter::new(
        StrategyKind::Arbitrage,
        run_mode(config.execution.mode),
        "Arbitrage",
        Some(config_path.clone()),
    );
    reporter.start(format!(
        "mode={:?} min_profit_usd={} max_position_per_trade={} kelly_sizing={}",
        config.execution.mode,
        config.strategy.min_profit_usd,
        config.strategy.max_position_per_trade,
        config.quant.apply_kelly_sizing
    ));

    // 主循环：带重连逻辑
    let mut reconnect_delay = Duration::from_secs(1);
    let max_reconnect_delay = Duration::from_secs(60);

    loop {
        match run_arbitrage_loop(&config, &scanner, &detector, &executor, &mut reporter).await {
            Ok(()) => {
                // 正常退出（通常不会发生）
                reporter.stop("arbitrage loop exited normally");
                break;
            }
            Err(e) => {
                error!(
                    "Arbitrage loop error: {}. Reconnecting in {}s...",
                    e,
                    reconnect_delay.as_secs()
                );
                reporter.warning(
                    "Arbitrage loop reconnecting",
                    format!("error={} retry_in={}s", e, reconnect_delay.as_secs()),
                );
                sleep(reconnect_delay).await;

                // 指数退避
                reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
            }
        }
    }

    Ok(())
}

/// 运行套利主循环
async fn run_arbitrage_loop(
    config: &Config,
    scanner: &Scanner,
    detector: &Arc<Detector>,
    executor: &Arc<Executor>,
    reporter: &mut PaperRunReporter,
) -> Result<()> {
    info!("Fetching initial market state via HTTP...");
    let initial_markets = scanner.scan().await?;
    let mut state = MarketState::new(initial_markets);

    let asset_ids = state.get_all_assets();
    info!(
        "Initial market state loaded. Tracking {} assets. Subscribing to Market WS...",
        asset_ids.len()
    );

    // 使用官方 SDK 的 WebSocket 客户端（带代理配置）
    let mut ws_config = WsConfig::default();
    ws_config.proxy_url = config.clob.proxy_url.clone();
    let ws_client = WsClient::new(
        "wss://ws-subscriptions-clob.polymarket.com",
        ws_config,
    )
    .context("Failed to create WebSocket client")?;
    let stream = ws_client
        .subscribe_orderbook(asset_ids)
        .context("Failed to subscribe to orderbook")?;

    info!("Arbitrage initialized and waiting for real-time WS events...");

    let mut stream = Box::pin(stream);
    while let Some(book_result) = stream.next().await {
        match book_result {
            Ok(book) => {
                match process_book_update(&mut state, detector, &book, |opportunity| {
                    let executor = Arc::clone(executor);
                    let opportunity = opportunity.clone();
                    async move { executor.execute(&opportunity).await }
                })
                .await
                {
                    Ok(outcome) => {
                        for event in outcome.events {
                            if event.kind == PaperEventKind::OpportunityDetected {
                                info!("Opportunity detected via WS: {}", event.detail);
                            } else if let Some(expected_edge) = event.expected_edge_usd {
                                info!(
                                    "Opportunity handled: kind={:?} expected_edge={} realized_pnl={:?}",
                                    event.kind, expected_edge, event.pnl_delta_usd
                                );
                            }
                            event.apply(reporter);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to process book update: {}", e);
                        reporter.error("Opportunity execution failed", e.to_string());
                    }
                }
            }
            Err(e) => {
                warn!("WebSocket error: {}", e);
                reporter.warning("WebSocket error", e.to_string());
                // 返回错误以触发重连
                return Err(e).context("WebSocket stream error");
            }
        }
    }

    // 流结束，返回错误以触发重连
    anyhow::bail!("WebSocket stream ended unexpectedly")
}

fn run_mode(mode: ExecutionMode) -> RunMode {
    match mode {
        ExecutionMode::Paper => RunMode::Paper,
        ExecutionMode::Live => RunMode::Live,
    }
}
