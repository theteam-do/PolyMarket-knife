//! 交易复制器 - 使用 poly-client

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use tracing::{info, instrument};

use crate::config::Config;
use crate::monitor::{TradeEvent, Side};

pub struct TradeCopier {
    client: PolyClient,
    config: Config,
}

impl TradeCopier {
    pub fn new(config: &Config) -> Self {
        let client = if config.polygon.private_key.is_empty() {
            PolyClient::new(&config.clob.host)
        } else {
            PolyClient::with_auth(&config.clob.host, &config.to_auth_config())
        };

        Self {
            client,
            config: config.clone(),
        }
    }

    #[instrument(skip(self), fields(trade = ?trade))]
    pub async fn copy(&self, trade: &TradeEvent) -> Result<()> {
        let size = self.calculate_copy_size(trade.size_usd);
        
        // 检查滑点
        let current_price = self.get_current_price(&trade.market_id).await?;
        let slippage = (current_price - Decimal::from_f64_retain(trade.price).unwrap()).abs() 
            / Decimal::from_f64_retain(trade.price).unwrap();
        
        let slippage_tolerance = Decimal::from_f64_retain(self.config.strategy.slippage_tolerance).unwrap();
        if slippage > slippage_tolerance {
            return Err(anyhow::anyhow!(
                "Slippage too high: {}% (max: {}%)",
                slippage * Decimal::from(100),
                slippage_tolerance * Decimal::from(100)
            ));
        }

        // 执行交易
        self.place_order(&trade.market_id, trade.side, size).await?;
        
        info!(
            "Copied trade: {} ${} @ {} (slippage: {}%)",
            if trade.side == Side::Buy { "BUY" } else { "SELL" },
            size,
            current_price,
            slippage * Decimal::from(100)
        );

        Ok(())
    }

    fn calculate_copy_size(&self, original_size: f64) -> Decimal {
        let copy_ratio = Decimal::from_f64_retain(self.config.strategy.copy_ratio).unwrap();
        let size = Decimal::from_f64_retain(original_size).unwrap() * copy_ratio;
        
        let min_size = Decimal::from_f64_retain(self.config.strategy.min_trade_size_usd).unwrap();
        let max_size = Decimal::from_f64_retain(self.config.strategy.max_trade_size_usd).unwrap();
        
        size.clamp(min_size, max_size)
    }

    async fn get_current_price(&self, _market_id: &str) -> Result<Decimal> {
        // TODO: 从 CLOB API 获取当前价格
        // 暂时返回模拟价格
        Ok(Decimal::from_f64_retain(0.50).unwrap())
    }

    async fn place_order(&self, _token_id: &str, side: Side, size: Decimal) -> Result<()> {
        // TODO: 在 CLOB 下单
        // 需要获取实际的 token_id
        
        let poly_side = match side {
            Side::Buy => PolySide::Buy,
            Side::Sell => PolySide::Sell,
        };

        // 模拟下单
        info!("Would place order: {:?} {} @ market", poly_side, size);
        
        Ok(())
    }
}
