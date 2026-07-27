//! Market Maker - 生产级主程序

use anyhow::{Context, Result};
use polymarket_client_sdk::clob::types::OrderStatusType;
use polymarket_client_sdk::clob::ws::{OrderMessage, TradeMessage};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};

mod api;
mod config;
mod executor;
mod metrics;
mod order_book;
mod quoting;
mod risk;
mod user_stream;

use config::Config;
use executor::{shares_for_target_notional, Executor};
use metrics::MetricsCollector;
use monitor::alerts::{AlertConfig, AlertManager};
use quoting::Quoter;
use risk::{OpenOrderSide, RiskManager};
use rust_decimal_macros::dec;
use user_stream::{spawn_user_stream, UserSyncEvent};

const PROCESSED_TRADE_WINDOW: usize = 10_000;

pub struct MarketMaker {
    config: Config,
    executor: Executor,
    quoter: Quoter,
    risk_manager: Arc<Mutex<RiskManager>>,
    metrics: Arc<MetricsCollector>,
    alert_manager: Arc<Mutex<AlertManager>>,
    processed_trades: HashSet<String>,
    processed_trade_order: VecDeque<String>,
    running: bool,
}

impl MarketMaker {
    pub async fn new(config: Config) -> Result<Self> {
        let executor = Executor::new(&config)?;
        let quoter = Quoter::new(&config.strategy);
        let risk_manager = Arc::new(Mutex::new(RiskManager::new(&config.risk)));
        let metrics = Arc::new(MetricsCollector::new());

        let alert_config = AlertConfig {
            daily_loss_threshold: dec!(500),
            daily_loss_warning_pct: 0.8,
            position_threshold: dec!(10000),
            api_error_rate_threshold: 0.1,
            latency_threshold_ms: 100.0,
            consecutive_loss_threshold: 5,
            cooldown_duration: std::time::Duration::from_secs(60),
        };
        let alert_manager = Arc::new(Mutex::new(AlertManager::new(alert_config)));

        info!("Market Maker initialized");
        info!("Monitoring {} markets", config.strategy.market_ids.len());
        info!(
            "Per-order target notional: ${}",
            config.strategy.order_size_usd
        );
        info!("Side mode: {:?}", config.strategy.side_mode);
        info!("Max daily loss: ${}", config.risk.max_loss_per_day);

        Ok(Self {
            config,
            executor,
            quoter,
            risk_manager,
            metrics,
            alert_manager,
            processed_trades: HashSet::new(),
            processed_trade_order: VecDeque::new(),
            running: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("Market Maker starting...");

        let metrics_clone = self.metrics.clone();
        let metrics_addr = self.config.strategy.metrics_bind_addr.clone();
        tokio::spawn(async move {
            if let Err(e) = start_metrics_server(metrics_clone, &metrics_addr).await {
                error!("Metrics server error: {}", e);
            }
        });

        let (user_tx, mut user_rx) = mpsc::channel(256);
        let user_stream_task = spawn_user_stream(&self.config, user_tx)?;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
            self.config.strategy.refresh_interval_ms,
        ));

        let mut tick_count = 0u64;
        while self.running {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutdown signal received");
                    self.stop();
                }
                _ = interval.tick() => {
                    match self.tick().await {
                        Ok(_) => {
                            tick_count += 1;
                            if tick_count.is_multiple_of(100) {
                                info!("Processed {} ticks", tick_count);
                            }
                        }
                        Err(e) => {
                            error!("Tick error: {}", e);
                        }
                    }
                }
                maybe_event = user_rx.recv() => {
                    match maybe_event {
                        Some(event) => self.handle_user_sync_event(event).await?,
                        None => anyhow::bail!("user sync channel closed unexpectedly"),
                    }
                }
            }
        }

        // CRIT-02: Always clear local state, regardless of cancel_all outcome
        match self.executor.cancel_all_orders().await {
            Ok(()) => info!("All orders cancelled successfully"),
            Err(e) => warn!("Failed to cancel all orders: {}", e),
        }
        {
            let mut risk = self.risk_manager.lock().await;
            risk.clear_open_orders();
        }

        // Wait briefly for user stream cancellation confirmations before aborting
        info!("Waiting for user stream cancellation confirmations...");
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        // Drain any remaining events
        while let Ok(Some(event)) = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            user_rx.recv(),
        ).await {
            if let UserSyncEvent::Order(_) = &event {
                let _ = self.handle_user_sync_event(event).await;
            }
        }

        user_stream_task.abort();

        info!("Market Maker stopped after {} ticks", tick_count);
        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        {
            let risk = self.risk_manager.lock().await;
            if !risk.can_trade() {
                warn!("Risk manager blocked trading");
                return Ok(());
            }

            let daily_pnl = risk.daily_pnl();
            let mut alert_mgr = self.alert_manager.lock().await;
            let alerts = alert_mgr
                .check_daily_loss(Decimal::from_f64_retain(daily_pnl).unwrap_or(Decimal::ZERO));
            for alert in alerts {
                warn!("🚨 ALERT: {}", alert.message);
            }
        }

        for market_id in self.config.strategy.market_ids.clone() {
            if let Err(e) = self.update_market(&market_id).await {
                error!("Failed to update market {}: {}", market_id, e);

                let mut alert_mgr = self.alert_manager.lock().await;
                let alerts = alert_mgr.record_order_failure(&market_id, &e.to_string());
                for alert in alerts {
                    error!("🚨 ALERT: {}", alert.message);
                }
            }
        }

        Ok(())
    }

    async fn update_market(&mut self, market_id: &str) -> Result<()> {
        let order_book = self.executor.fetch_orderbook(market_id).await?;
        let mid_price = order_book
            .mid_price()
            .with_context(|| format!("order book for {} is missing best bid/ask", market_id))?;
        let spread = order_book.spread().unwrap_or(0.0);
        let spread_bps = order_book.spread_bps().unwrap_or(0);
        let top_bid_size = order_book
            .bids
            .first()
            .map(|level| level.size)
            .unwrap_or(0.0);
        let top_ask_size = order_book
            .asks
            .first()
            .map(|level| level.size)
            .unwrap_or(0.0);
        info!(
            "Market {} top-of-book: mid={:.4} spread={:.4} ({} bps) bid_size={:.2} ask_size={:.2}",
            order_book.token_id, mid_price, spread, spread_bps, top_bid_size, top_ask_size
        );

        let target_notional_usd = self.quoter.order_size();
        let position_signal = {
            let risk = self.risk_manager.lock().await;
            risk.inventory_skew_signal(market_id)
        };
        let (bid, ask) = self
            .quoter
            .calculate_quotes_with_position(mid_price, position_signal);

        let planned_buy = if self.config.strategy.side_mode.allows_buy() {
            let (shares, notional_usd) = shares_for_target_notional(target_notional_usd, bid)
                .with_context(|| format!("failed to compute bid share size for {}", market_id))?;
            Some((bid, shares, notional_usd))
        } else {
            None
        };
        let planned_sell = if self.config.strategy.side_mode.allows_sell() {
            let (shares, notional_usd) = shares_for_target_notional(target_notional_usd, ask)
                .with_context(|| format!("failed to compute ask share size for {}", market_id))?;
            Some((ask, shares, notional_usd))
        } else {
            None
        };

        let proposed_notionals: Vec<f64> = planned_buy
            .iter()
            .map(|(_, _, notional_usd)| *notional_usd)
            .chain(
                planned_sell
                    .iter()
                    .map(|(_, _, notional_usd)| *notional_usd),
            )
            .collect();

        {
            let risk = self.risk_manager.lock().await;
            if !risk.can_place_orders(market_id, &proposed_notionals) {
                warn!(
                    "Risk check failed for market {} with proposed_open_notional={:.4}",
                    market_id,
                    proposed_notionals.iter().sum::<f64>()
                );
                return Ok(());
            }
        }

        if let Some((price, shares, notional_usd)) = &planned_buy {
            info!(
                "Prepared BUY quote for {}: price={:.4} shares={} notional=${:.4}",
                market_id, price, shares, notional_usd
            );
        }
        if let Some((price, shares, notional_usd)) = &planned_sell {
            info!(
                "Prepared SELL quote for {}: price={:.4} shares={} notional=${:.4}",
                market_id, price, shares, notional_usd
            );
        }

        let buy_request = planned_buy
            .as_ref()
            .map(|(price, shares, _)| (*price, *shares));
        let sell_request = planned_sell
            .as_ref()
            .map(|(price, shares, _)| (*price, *shares));
        let (buy_result, sell_result) = self
            .executor
            .place_orders(market_id, buy_request, sell_request)
            .await?;

        let planned_count =
            usize::from(planned_buy.is_some()) + usize::from(planned_sell.is_some());
        let success_count = usize::from(buy_result.is_some()) + usize::from(sell_result.is_some());
        let failure_count = planned_count.saturating_sub(success_count);

        if success_count == planned_count {
            self.metrics.record_placed(success_count as u64);
            let mut risk = self.risk_manager.lock().await;
            if let (Some((bid_price, bid_shares, _)), Some(buy_id)) = (&planned_buy, &buy_result) {
                risk.reserve_open_order(
                    buy_id,
                    market_id,
                    OpenOrderSide::Buy,
                    *bid_price,
                    decimal_to_f64(bid_shares)?,
                );
            }
            if let (Some((ask_price, ask_shares, _)), Some(sell_id)) = (&planned_sell, &sell_result)
            {
                risk.reserve_open_order(
                    sell_id,
                    market_id,
                    OpenOrderSide::Sell,
                    *ask_price,
                    decimal_to_f64(ask_shares)?,
                );
            }
            info!(
                "Orders accepted for {}: buy={:?}, sell={:?}",
                market_id, buy_result, sell_result
            );
            info!(
                "Risk state: market={} open_orders={} open_notional={:.4} market_open_notional={:.4}",
                market_id,
                risk.open_order_count(),
                risk.total_open_order_value(),
                risk.market_open_order_value(market_id)
            );
        } else if success_count == 0 {
            warn!("All planned orders failed for market {}", market_id);
            self.metrics.record_failed(failure_count as u64);
        } else {
            self.metrics.record_placed(success_count as u64);
            self.metrics.record_failed(failure_count as u64);
            if let (Some((bid_price, bid_shares, _)), Some(buy_id)) = (&planned_buy, &buy_result) {
                self.handle_partial_order(
                    market_id,
                    buy_id,
                    OpenOrderSide::Buy,
                    *bid_price,
                    decimal_to_f64(bid_shares)?,
                )
                .await;
            }
            if let (Some((ask_price, ask_shares, _)), Some(sell_id)) = (&planned_sell, &sell_result)
            {
                self.handle_partial_order(
                    market_id,
                    sell_id,
                    OpenOrderSide::Sell,
                    *ask_price,
                    decimal_to_f64(ask_shares)?,
                )
                .await;
            }
        }

        Ok(())
    }

    async fn handle_partial_order(
        &mut self,
        market_id: &str,
        order_id: &str,
        side: OpenOrderSide,
        price: f64,
        shares: f64,
    ) {
        warn!(
            "Partial order placement for market {}. Cancelling surviving order {}",
            market_id, order_id
        );

        {
            let mut risk = self.risk_manager.lock().await;
            risk.reserve_open_order(order_id, market_id, side, price, shares);
        }

        match self.executor.cancel_orders(order_id).await {
            Ok(()) => {
                info!(
                    "Cancellation accepted for {}. Waiting for user-stream confirmation before releasing reservation",
                    order_id
                );
            }
            Err(e) => {
                warn!("Failed to cancel surviving order {}: {}", order_id, e);
            }
        }
    }

    async fn handle_user_sync_event(&mut self, event: UserSyncEvent) -> Result<()> {
        match event {
            UserSyncEvent::Order(order) => self.handle_order_event(order).await,
            UserSyncEvent::Trade(trade) => self.handle_trade_event(trade).await,
            UserSyncEvent::StreamClosed(reason) => {
                anyhow::bail!("user sync stream closed: {}", reason)
            }
        }
    }

    async fn handle_order_event(&mut self, order: OrderMessage) -> Result<()> {
        let cancelled = order
            .status
            .as_ref()
            .is_some_and(|status| matches!(status, OrderStatusType::Canceled));
        if !cancelled {
            return Ok(());
        }

        let mut risk = self.risk_manager.lock().await;
        if let Some(released_notional) = risk.release_open_order(&order.id) {
            self.metrics.record_cancelled(1);
            info!(
                "Released reservation from cancellation event: order_id={} market={} released_notional=${:.4}",
                order.id,
                order.market,
                released_notional
            );
        }

        Ok(())
    }

    async fn handle_trade_event(&mut self, trade: TradeMessage) -> Result<()> {
        if self.record_trade_if_new(&trade.id) {
            return Ok(());
        }

        let trade_price = decimal_to_f64(&trade.price)?;
        let trade_size = decimal_to_f64(&trade.size)?;
        let mut fill_candidates = Vec::new();

        if let Some(taker_order_id) = &trade.taker_order_id {
            fill_candidates.push((taker_order_id.clone(), trade_size, trade_price));
        }
        for maker in &trade.maker_orders {
            fill_candidates.push((
                maker.order_id.clone(),
                decimal_to_f64(&maker.matched_amount)?,
                decimal_to_f64(&maker.price)?,
            ));
        }

        let mut seen_order_ids = HashSet::new();
        let mut effects = Vec::new();
        {
            let mut risk = self.risk_manager.lock().await;
            for (order_id, fill_shares, fill_price) in fill_candidates {
                if !seen_order_ids.insert(order_id.clone()) {
                    continue;
                }
                if let Some(effect) = risk.apply_fill(&order_id, fill_shares, fill_price) {
                    effects.push(effect);
                }
            }
        }

        for effect in effects {
            if effect.order_completed {
                self.metrics.record_filled(1);
            }
            self.metrics.record_volume(
                Decimal::from_f64_retain(effect.fill_notional_usd).unwrap_or(Decimal::ZERO),
            );
            if effect.realized_pnl_delta.abs() > 1e-9 {
                self.metrics.record_pnl(
                    Decimal::from_f64_retain(effect.realized_pnl_delta).unwrap_or(Decimal::ZERO),
                );
            }
            info!(
                "Applied fill from user stream: trade_id={} order_id={} market={} shares={:.4} price={:.6} notional=${:.4} realized_pnl_delta=${:.4} remaining_open_notional=${:.4} net_position_shares={:.4}",
                trade.id,
                effect.order_id,
                effect.market_id,
                effect.fill_shares,
                effect.fill_price,
                effect.fill_notional_usd,
                effect.realized_pnl_delta,
                effect.remaining_open_notional_usd,
                effect.net_position_shares
            );
        }

        Ok(())
    }

    fn record_trade_if_new(&mut self, trade_id: &str) -> bool {
        if self.processed_trades.contains(trade_id) {
            return true;
        }

        self.processed_trades.insert(trade_id.to_string());
        self.processed_trade_order.push_back(trade_id.to_string());
        while self.processed_trade_order.len() > PROCESSED_TRADE_WINDOW {
            if let Some(evicted) = self.processed_trade_order.pop_front() {
                self.processed_trades.remove(&evicted);
            }
        }
        false
    }

    pub fn stop(&mut self) {
        self.running = false;
        if let Ok(mut risk) = self.risk_manager.try_lock() {
            risk.stop_trading();
            risk.reset_daily();
            risk.resume_trading();
            info!("Risk reset on stop, daily_pnl={:.2}", risk.daily_pnl());
        }
        self.metrics.reset_daily();
        info!("Stopping Market Maker...");
    }
}

async fn start_metrics_server(metrics: Arc<MetricsCollector>, bind_addr: &str) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::time::timeout;

    let addr: SocketAddr = bind_addr
        .parse()
        .with_context(|| format!("invalid metrics bind address: {}", bind_addr))?;
    let listener = TcpListener::bind(addr).await?;

    info!("Metrics server listening on http://{}", addr);

    loop {
        // Accept with timeout to allow graceful shutdown (accept is cancelable via drop)
        let accept_result = timeout(Duration::from_secs(30), listener.accept()).await;
        let (mut socket, _) = match accept_result {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                error!("Metrics server accept error: {}", e);
                continue;
            }
            Err(_) => {
                // Timeout is normal, just keep listening
                continue;
            }
        };

        let metrics = metrics.clone();

        tokio::spawn(async move {
            // Read the HTTP request with timeout
            let mut buf = [0u8; 1024];
            let read_result = timeout(Duration::from_secs(5), socket.read(&mut buf)).await;
            if read_result.is_err() {
                // Client took too long to send request; close connection
                return;
            }

            let prometheus_output = metrics.export_prometheus();
            let body = prometheus_output.as_bytes();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );

            // Write response with timeout
            let _ = timeout(Duration::from_secs(5), async {
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.write_all(body).await;
            })
            .await;
        });
    }
}

fn decimal_to_f64(value: &Decimal) -> Result<f64> {
    value
        .to_f64()
        .with_context(|| format!("failed to convert Decimal {} to f64", value))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("market_maker=info".parse()?),
        )
        .json()
        .init();

    info!("Market Maker starting up...");

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/market-maker.toml".to_string());

    let config = Config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path))?;

    let mut mm = MarketMaker::new(config).await?;
    mm.run().await?;

    Ok(())
}
