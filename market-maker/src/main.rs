//! Market Maker - 返佣做市策略
//! 
//! 核心逻辑：双边挂单赚返佣 + 价差
//! 延迟目标：<100ms 撤单重下

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error, instrument};

mod config;
mod order_book;
mod quoting;
mod risk;
mod executor;
mod polychain;

use config::Config;
use order_book::OrderBook;
use quoting::Quoter;
use risk::RiskManager;
use executor::Executor;

pub struct MarketMaker {
    config: Config,
    order_book: Arc<Mutex<OrderBook>>,
    quoter: Quoter,
    risk_manager: Arc<Mutex<RiskManager>>,
    executor: Option<Executor>,
    running: bool,
}

impl MarketMaker {
    pub async fn new(config: Config) -> Result<Self> {
        let executor = Executor::new(&config).await.ok();
        
        Ok(Self {
            order_book: Arc::new(Mutex::new(OrderBook::new())),
            quoter: Quoter::new(&config.strategy),
            risk_manager: Arc::new(Mutex::new(RiskManager::new(&config.risk))),
            executor,
            config,
            running: false,
        })
    }

    #[instrument(skip(self), fields(name = "market_maker"))]
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        info!("Market Maker starting...");

        // 启动订单簿更新循环
        let ob_clone = Arc::clone(&self.order_book);
        let executor_clone = self.executor.as_ref().unwrap().clone();
        let market_ids = self.config.strategy.market_ids.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::update_orderbook_loop(ob_clone, executor_clone, market_ids).await {
                error!("OrderBook update error: {}", e);
            }
        });

        // 主循环 - 单线程事件驱动
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_millis(self.config.strategy.refresh_interval_ms)
        );

        while self.running {
            interval.tick().await;
            
            if let Err(e) = self.tick().await {
                error!("Tick error: {}", e);
            }
        }

        Ok(())
    }

    async fn update_orderbook_loop(
        order_book: Arc<Mutex<OrderBook>>,
        executor: Executor,
        market_ids: Vec<String>,
    ) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        
        loop {
            interval.tick().await;
            
            for market_id in &market_ids {
                match executor.fetch_orderbook(market_id).await {
                    Ok(ob) => {
                        let mut book = order_book.lock().await;
                        // Convert poly-client OrderBook to local OrderBookLevels
                        let levels = order_book::OrderBookLevels {
                            bids: ob.bids.into_iter().map(|l| order_book::Level { 
                                price: l.price.to_string().parse().unwrap_or(0.0), 
                                size: l.size.to_string().parse().unwrap_or(0.0) 
                            }).collect(),
                            asks: ob.asks.into_iter().map(|l| order_book::Level { 
                                price: l.price.to_string().parse().unwrap_or(0.0), 
                                size: l.size.to_string().parse().unwrap_or(0.0) 
                            }).collect(),
                        };
                        book.update(market_id, levels);
                    }
                    Err(e) => {
                        warn!("Failed to fetch orderbook for {}: {}", market_id, e);
                    }
                }
            }
        }
    }

    #[instrument(skip(self))]
    async fn tick(&mut self) -> Result<()> {
        // 1. 风控检查
        {
            let risk = self.risk_manager.lock().await;
            if !risk.can_trade() {
                warn!("Risk manager blocked trading");
                return Ok(());
            }
        }

        // 2. 对每个市场计算新报价
        let market_ids = self.config.strategy.market_ids.clone();
        for market_id in market_ids {
            if let Err(e) = self.update_quotes(&market_id).await {
                error!("Failed to update quotes for {}: {}", market_id, e);
            }
        }

        Ok(())
    }

    #[instrument(skip(self), fields(market_id = %market_id))]
    async fn update_quotes(&mut self, market_id: &str) -> Result<()> {
        let book = self.order_book.lock().await;
        let Some(market_book) = book.get_market(market_id) else {
            return Ok(());
        };

        // 3. 计算新报价
        let (bid_price, ask_price) = self.quoter.calculate_quotes(market_book);

        // 4. 检查是否需要重新报价
        let needs_requote = self.needs_requote(market_id, bid_price, ask_price).await;
        
        drop(book);

        if needs_requote {
            self.requote(market_id, bid_price, ask_price).await?;
        }

        Ok(())
    }

    async fn needs_requote(&self, market_id: &str, bid: f64, ask: f64) -> bool {
        // 检查当前挂单是否偏离最优价格
        let book = self.order_book.lock().await;
        let Some(market_book) = book.get_market(market_id) else {
            return true;
        };

        let Some(best_bid) = market_book.best_bid else {
            return true;
        };
        let Some(best_ask) = market_book.best_ask else {
            return true;
        };

        // 如果我们的报价不在最优位置，需要重新报价
        let threshold = 0.01; // 1% 阈值
        (bid - best_bid).abs() / best_bid > threshold ||
        (ask - best_ask).abs() / best_ask > threshold
    }

    #[instrument(skip(self), fields(market_id = %market_id))]
    async fn requote(&mut self, market_id: &str, bid: f64, ask: f64) -> Result<()> {
        let risk = self.risk_manager.lock().await;
        
        // 风控检查后下新单
        if risk.can_place_order(market_id, bid, ask) {
            // 取消旧订单
            self.executor.as_ref().unwrap().cancel_orders(market_id).await?;
            
            // 下新订单
            self.executor.as_ref().unwrap().place_orders(market_id, bid, ask).await?;
            
            info!("Requoted: bid={}, ask={}", bid, ask);
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("market_maker=info".parse()?)
        )
        .json()
        .init();

    // 加载配置
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/market-maker.toml".to_string());
    
    let config = Config::load(&config_path)
        .context("Failed to load config")?;

    // 创建并运行做市商
    let mut mm = MarketMaker::new(config).await?;
    
    // 处理信号
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutting down...");
    });

    mm.run().await
}
