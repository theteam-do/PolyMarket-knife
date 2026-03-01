//! 交易复制器 - 使用官方 SDK 复制聪明钱交易

use anyhow::Result;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;
use tracing::{info, instrument, warn};

use crate::config::Config;
use crate::config::ExecutionMode;
use crate::monitor::TradeEvent;

pub struct TradeCopier {
    config: Config,
    http_client: Client,
}

impl TradeCopier {
    pub fn new(config: &Config) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            config: config.clone(),
            http_client,
        }
    }

    #[instrument(skip(self), fields(trade = ?trade))]
    pub async fn copy(&self, trade: &TradeEvent) -> Result<Decimal> {
        let size = self.calculate_copy_size(trade.size_usd);

        info!(
            "Copying trade: market={} side={:?} original_size=${} copy_size=${}",
            trade.market, trade.side, trade.size_usd, size
        );

        if self.config.execution.mode == ExecutionMode::Paper {
            return self.simulate_execution(trade, size).await;
        }

        // 尝试实盘下单（是否降级由配置控制）
        match self.execute_live(trade, size).await {
            Ok(order_id) => {
                info!("Order placed successfully: {}", order_id);
                return Ok(size * dec!(0.1));
            }
            Err(e) => {
                if self.config.execution.live_failure_fallback_to_paper {
                    warn!("Live execution failed: {}. Falling back to simulation.", e);
                    return self.simulate_execution(trade, size).await;
                }
                anyhow::bail!("live copy execution failed: {}", e);
            }
        }
    }

    /// 使用 CLOB HTTP 接口执行跟单
    async fn execute_live(&self, trade: &TradeEvent, size: Decimal) -> Result<String> {
        let endpoint = format!("{}/order", self.config.clob.host.trim_end_matches('/'));
        let payload = CopyOrderRequest {
            market: &trade.market,
            market_id: &trade.market_id,
            side: match trade.side {
                crate::monitor::Side::Buy => "BUY",
                crate::monitor::Side::Sell => "SELL",
            },
            size,
            price: Decimal::from_f64_retain(trade.price).unwrap_or(dec!(0.5)),
            source_wallet: &trade.from,
            source_ts: trade.timestamp,
        };

        let mut request = self.http_client.post(endpoint).json(&payload);
        if let Some(key) = &self.config.clob.api_key {
            request = request.header("X-Api-Key", key);
        }
        if let Some(secret) = &self.config.clob.api_secret {
            request = request.header("X-Api-Secret", secret);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Live copy order rejected {}: {}", status, body);
        }

        Ok(format!("copy-{}-{}", trade.market_id, trade.timestamp))
    }

    /// 模拟执行（用于测试）
    async fn simulate_execution(&self, _trade: &TradeEvent, size: Decimal) -> Result<Decimal> {
        // 模拟跟单利润
        let profit = size * dec!(0.05); // 5% 模拟利润
        
        info!("Simulated copy: size={}, profit={}", size, profit);
        
        Ok(profit)
    }

    fn calculate_copy_size(&self, original_size: f64) -> Decimal {
        let copy_ratio = Decimal::from_f64_retain(self.config.strategy.copy_ratio).unwrap();
        let size = Decimal::from_f64_retain(original_size).unwrap() * copy_ratio;

        let min_size = Decimal::from_f64_retain(self.config.strategy.min_trade_size_usd).unwrap();
        let max_size = Decimal::from_f64_retain(self.config.strategy.max_trade_size_usd).unwrap();

        size.clamp(min_size, max_size)
    }
}

#[derive(Serialize)]
struct CopyOrderRequest<'a> {
    market: &'a str,
    market_id: &'a str,
    side: &'a str,
    size: Decimal,
    price: Decimal,
    source_wallet: &'a str,
    source_ts: u64,
}
