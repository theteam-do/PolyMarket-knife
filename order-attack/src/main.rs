//! Order Attack - 订单攻击策略
//!
//! ⚠️⚠️⚠️ 高风险警告：仅供测试网学习使用 ⚠️⚠️⚠️
//!
//! 主网使用可能导致：
//! - 永久封号
//! - 法律诉讼
//! - 社区抵制

use anyhow::{anyhow, Context, Result};
use tracing::{error, info, instrument, warn};

mod attacker;
mod config;
mod monitor;
mod scanner;

use attacker::AttackExecutor;
use config::Config;
use monitor::OrderbookMonitor;
use scanner::TargetScanner;

pub struct OrderAttacker {
    config: Config,
    scanner: TargetScanner,
    attacker: AttackExecutor,
    monitor: OrderbookMonitor,
    attacks_today: u32,
    running: bool,
}

impl OrderAttacker {
    pub fn new(config: Config) -> Self {
        Self {
            scanner: TargetScanner::new(&config.strategy),
            attacker: AttackExecutor::new(&config),
            monitor: OrderbookMonitor::new(&config),
            attacks_today: 0,
            config,
            running: false,
        }
    }

    #[instrument(skip(self), fields(name = "order_attack"))]
    pub async fn run(&mut self) -> Result<()> {
        // ⚠️ 安全检查
        if !self.config.warning.acknowledged {
            return Err(anyhow!(
                "⚠️ You must acknowledge the risks in config file before running!\n\
                 Set [warning] acknowledged = true only if you understand:\n\
                 - This is for TESTNET ONLY\n\
                 - Mainnet use may result in permanent ban\n\
                 - Legal risks may apply"
            ));
        }

        if !self.config.warning.testnet_only {
            return Err(anyhow!(
                "⚠️ This strategy is ONLY for testnet!\n\
                 Change testnet_only to false is PROHIBITED!"
            ));
        }

        self.running = true;
        warn!("⚠️ Order Attacker starting on TESTNET only ⚠️");

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.strategy.cooldown_seconds,
        ));

        while self.running {
            interval.tick().await;

            // 检查攻击次数限制
            if self.attacks_today >= self.config.strategy.max_attacks_per_day {
                warn!("Daily attack limit reached ({})", self.attacks_today);
                continue;
            }

            if let Err(e) = self.tick().await {
                error!("Tick error: {}", e);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn tick(&mut self) -> Result<()> {
        // 1. 扫描目标
        let targets = self.scanner.scan().await?;

        // 每次只攻击一个目标
        if let Some(target) = targets.first() {
            info!("🎯 Target identified: {:?}", target);

            // 2. 执行攻击
            match self.attacker.execute(target).await {
                Ok(_) => {
                    self.attacks_today += 1;
                    info!("✅ Attack executed successfully");

                    // 3. 等待订单簿清空
                    if self.monitor.wait_for_clearing(&target.market).await {
                        info!("📊 Orderbook cleared, monopoly opportunity detected");

                        // 4. 垄断交易
                        match self.trade_monopoly(&target.market).await {
                            Ok(_) => {
                                info!("💰 Monopoly trade executed");
                            }
                            Err(e) => {
                                warn!("❌ Monopoly trade failed: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("❌ Attack failed: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn trade_monopoly(&self, _market: &str) -> Result<()> {
        // TODO: 挂出垄断价差订单
        // 在流动性真空时挂出大幅价差

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
                .add_directive("order_attack=info".parse()?),
        )
        .json()
        .init();

    // 加载配置
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/order-attack.toml".to_string());

    let config = Config::load(&config_path).context("Failed to load config")?;

    let mut attacker = OrderAttacker::new(config);

    // 处理信号
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Shutting down...");
    });

    attacker.run().await
}
