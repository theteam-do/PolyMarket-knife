//! Market Maker - 生产级主程序

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

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
            interval.tick().await;

            match self.tick().await {
                Ok(_) => {
                    tick_count += 1;
                    if tick_count % 100 == 0 {
                        info!("Processed {} ticks", tick_count);
                    }
                }
                Err(e) => {
                    error!("Tick error: {}", e);
                    self.metrics.record_order(OrderStatus::Failed);
                }
            }
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
        for market_id in &self.config.strategy.market_ids.clone() {
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
        let _order_book = self.executor.fetch_orderbook(market_id).await?;

        // TODO: 计算新报价并下单
        // let (bid, ask) = self.quoter.calculate_quotes(&order_book);
        // let (buy_id, sell_id) = self.executor.place_orders(market_id, bid, ask).await?;
        
        // 记录指标
        self.metrics.record_order(OrderStatus::Filled);

        Ok(())
    }

    /// 停止做市商
    pub fn stop(&mut self) {
        self.running = false;
        info!("Stopping Market Maker...");
    }
}

/// 启动监控指标 HTTP 服务器
async fn start_metrics_server(metrics: Arc<MetricsCollector>) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let addr: SocketAddr = "0.0.0.0:9090".parse().unwrap();
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

    // 处理关闭信号
    let shutdown_handle = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutdown signal received");
    });

    mm.run().await?;

    shutdown_handle.abort();

    Ok(())
}
