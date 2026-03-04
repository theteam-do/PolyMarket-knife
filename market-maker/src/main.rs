//! Market Maker - 生产级主程序

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

mod api;
mod config;
mod executor;
mod metrics;
mod order_book;
mod quoting;
mod risk;

use config::Config;
use executor::Executor;
use metrics::{MetricsCollector, OrderStatus};
use quoting::Quoter;
use risk::RiskManager;

/// 做市商主结构
pub struct MarketMaker {
    config: Config,
    executor: Executor,
    quoter: Quoter,
    risk_manager: Arc<Mutex<RiskManager>>,
    metrics: Arc<MetricsCollector>,
    running: bool,
}

impl MarketMaker {
    /// 创建新的做市商
    pub async fn new(config: Config) -> Result<Self> {
        let executor = Executor::new(&config)?;
        let quoter = Quoter::new(&config.strategy);
        let risk_manager = Arc::new(Mutex::new(RiskManager::new(&config.risk)));
        let metrics = Arc::new(MetricsCollector::new());

        info!("Market Maker initialized");
        info!("Monitoring {} markets", config.strategy.market_ids.len());
        info!("Order size: ${}", config.strategy.order_size_usd);
        info!("Max daily loss: ${}", config.risk.max_loss_per_day);

        Ok(Self {
            config,
            executor,
            quoter,
            risk_manager,
            metrics,
            running: false,
        })
    }

    /// 运行做市商
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("Market Maker starting...");

        // 启动监控指标 HTTP 服务器
        let metrics_clone = self.metrics.clone();
        tokio::spawn(async move {
            if let Err(e) = start_metrics_server(metrics_clone).await {
                error!("Metrics server error: {}", e);
            }
        });

        // 主循环
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
                            self.metrics.record_order(OrderStatus::Failed);
                        }
                    }
                }
            }
        }

        if let Err(e) = self.executor.cancel_all_orders().await {
            warn!("Failed to cancel all orders during shutdown: {}", e);
        }

        info!("Market Maker stopped after {} ticks", tick_count);
        Ok(())
    }

    /// 单次迭代
    async fn tick(&mut self) -> Result<()> {
        // 风控检查
        {
            let risk = self.risk_manager.lock().await;
            if !risk.can_trade() {
                warn!("Risk manager blocked trading");
                return Ok(());
            }
        }

        // 对每个市场更新报价
        for market_id in self.config.strategy.market_ids.clone() {
            if let Err(e) = self.update_market(&market_id).await {
                error!("Failed to update market {}: {}", market_id, e);
                self.metrics.record_order(OrderStatus::Failed);
            }
        }

        Ok(())
    }

    /// 更新市场报价
    async fn update_market(&mut self, market_id: &str) -> Result<()> {
        // 获取订单簿
        let order_book = self.executor.fetch_orderbook(market_id).await?;

        // 计算中间价
        let mid_price = order_book.mid_price().unwrap_or(0.50);
        let spread = order_book.spread().unwrap_or(0.0);
        let spread_bps = order_book.spread_bps().unwrap_or(0);
        let _mid_decimal = order_book.mid_price_decimal();
        let top_bid_size = order_book.bids.first().map(|l| l.size).unwrap_or(0.0);
        let top_ask_size = order_book.asks.first().map(|l| l.size).unwrap_or(0.0);
        info!(
            "Market {} top-of-book: mid={:.4} spread={:.4} ({} bps) bid_size={:.2} ask_size={:.2}",
            order_book.token_id,
            mid_price,
            spread,
            spread_bps,
            top_bid_size,
            top_ask_size
        );

        // 计算新报价
        let (bid, ask) = self.quoter.calculate_quotes(mid_price);
        let order_size = self.quoter.order_size();

        // 风控检查
        {
            let risk = self.risk_manager.lock().await;
            if !risk.can_place_order(market_id, order_size) {
                warn!("Risk check failed for market {}", market_id);
                return Ok(());
            }
        }

        // 下双边订单
        match self.executor.place_orders(market_id, bid, ask).await? {
            (Some(buy_id), Some(sell_id)) => {
                info!("Orders placed for {}: buy={}, sell={}", market_id, buy_id, sell_id);
                self.metrics.record_order(OrderStatus::Filled);
                self.metrics
                    .record_volume(Decimal::from_f64_retain(order_size).unwrap_or(Decimal::ZERO) * Decimal::from(2));
                self.metrics.record_pnl(Decimal::ZERO);
                
                // 更新风控持仓
                let mut risk = self.risk_manager.lock().await;
                risk.update_position(market_id, order_size, order_size, mid_price);
                risk.update_volume(order_size * 2.0);
                info!(
                    "Risk state: market={} total_position={:.2} daily_volume={:.2}",
                    market_id,
                    risk.total_position_value(),
                    risk.daily_volume()
                );
                if let Some(pos) = risk.get_position(market_id) {
                    info!(
                        "Position {} => yes={:.2} no={:.2} avg_cost={:.4}",
                        market_id,
                        pos.yes_size,
                        pos.no_size,
                        pos.avg_cost
                    );
                }
            }
            (None, None) => {
                warn!("Both orders failed for market {}", market_id);
                self.metrics.record_order(OrderStatus::Failed);
                let pnl_loss = Decimal::from_f64_retain(0.01).unwrap_or(Decimal::new(1, 2));
                let mut risk = self.risk_manager.lock().await;
                risk.update_pnl(-pnl_loss.to_f64().unwrap_or(0.01));
                self.metrics.record_pnl(-pnl_loss);
            }
            (Some(order_id), None) | (None, Some(order_id)) => {
                warn!("Partial fill for market {}. Cancelling surviving order {}", market_id, order_id);
                self.metrics.record_order(OrderStatus::Cancelled);
                if let Err(e) = self.executor.cancel_orders(&order_id).await {
                    warn!("Failed to cancel surviving order {}: {}", order_id, e);
                }
            }
        }

        Ok(())
    }

    /// 停止做市商
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

/// 启动监控指标 HTTP 服务器
async fn start_metrics_server(metrics: Arc<MetricsCollector>) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let addr: SocketAddr = "0.0.0.0:9090"
        .parse()
        .expect("Failed to parse metrics server address");
    let listener = TcpListener::bind(addr).await?;
    
    info!("Metrics server listening on http://{}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let metrics = metrics.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;

            // 简单的 HTTP 响应
            let prometheus_output = metrics.export_prometheus();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                prometheus_output.len(),
                prometheus_output
            );

            let _ = socket.write_all(response.as_bytes()).await;
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("market_maker=info".parse()?),
        )
        .json()
        .init();

    info!("Market Maker starting up...");

    // 加载配置
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/market-maker.toml".to_string());

    let config = Config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path))?;

    // 创建并运行做市商
    let mut mm = MarketMaker::new(config).await?;

    mm.run().await?;

    Ok(())
}
