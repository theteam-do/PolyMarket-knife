//! 订单执行器

use anyhow::Result;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;
use tracing::{info, instrument, warn};

use crate::config::Config;
use crate::signal::Signal;

/// 订单执行器
pub struct Executor {
    config: Config,
    http_client: Client,
}

impl Executor {
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

    #[instrument(skip(self), fields(signal = ?signal))]
    pub async fn execute(&self, signal: &Signal) -> Result<Decimal> {
        let position = self.calculate_position(signal.confidence());

        info!(
            "Executing order: symbol={} side={:?} confidence={:.2} position=${}",
            signal.symbol(),
            signal,
            signal.confidence(),
            position
        );

        // 尝试实盘下单，失败则降级到模拟
        match self.execute_live(signal, position).await {
            Ok(profit) => return Ok(profit),
            Err(e) => {
                warn!("Live execution failed: {}. Falling back to simulation.", e);
            }
        }
        
        // 模拟执行（用于测试/降级）
        self.simulate_execution(signal, position).await
    }

    async fn execute_live(&self, signal: &Signal, position: Decimal) -> Result<Decimal> {
        let endpoint = format!("{}/order", self.config.clob.host.trim_end_matches('/'));
        let (side, symbol) = match signal {
            Signal::Buy { symbol, .. } => ("BUY", symbol.as_str()),
            Signal::Sell { symbol, .. } => ("SELL", symbol.as_str()),
        };

        let payload = VolOrderRequest {
            symbol,
            side,
            size: position,
            order_type: "market",
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
            anyhow::bail!("Live order rejected {}: {}", status, body);
        }

        Ok(dec!(0))
    }

    /// 模拟执行（用于测试）
    async fn simulate_execution(&self, signal: &Signal, position: Decimal) -> Result<Decimal> {
        // 模拟交易利润：基于信号置信度
        let base_profit = position * dec!(0.05); // 5% 基础利润
        let confidence_multiplier = Decimal::from_f64_retain(signal.confidence()).unwrap_or(dec!(0.5));
        let profit = base_profit * confidence_multiplier * dec!(2.0);
        
        info!("Simulated execution: position={}, profit={}", position, profit);
        
        Ok(profit)
    }

    fn calculate_position(&self, confidence: f64) -> Decimal {
        let base = Decimal::from_f64_retain(self.config.strategy.base_position_usd).unwrap();
        let max = Decimal::from_f64_retain(self.config.strategy.max_position_usd).unwrap();

        if confidence >= self.config.strategy.confidence_high {
            max
        } else if confidence >= 0.6 {
            max * dec!(0.3)
        } else {
            base
        }
    }
}

#[derive(Serialize)]
struct VolOrderRequest<'a> {
    symbol: &'a str,
    side: &'a str,
    size: Decimal,
    order_type: &'a str,
}
