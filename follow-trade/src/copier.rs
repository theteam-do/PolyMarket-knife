//! 交易复制器 - 使用官方 SDK 复制聪明钱交易

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{info, instrument, warn};
use std::str::FromStr;

use polymarket_client_sdk::clob::{Client as ClobSdkClient, Config as ClobConfig};
use polymarket_client_sdk::clob::types::{Side as SdkSide, OrderType as SdkOrderType};
use polymarket_client_sdk::types::U256;
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use alloy::primitives::ChainId;

use crate::config::Config;
use crate::config::ExecutionMode;
use crate::monitor::TradeEvent;

const POLYGON_CHAIN_ID: ChainId = 137;

pub struct TradeCopier {
    config: Config,
}

impl TradeCopier {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
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
                // Return estimated profit or size based on strategy
                return Ok(size * dec!(0.05)); // 5% mock profit placeholder
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

    /// 使用 CLOB API SDK 执行跟单
    async fn execute_live(&self, trade: &TradeEvent, size: Decimal) -> Result<String> {
        let private_key = self.config.clob.api_secret.as_deref()
            .or(Some(&self.config.polygon.private_key))
            .context("Private key not configured for live execution")?;
        
        // 解析私钥并创建签名器
        let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
        let signer = LocalSigner::from_str(pk)
            .context("Failed to parse private key")?
            .with_chain_id(Some(POLYGON_CHAIN_ID));
        
        let host = self.config.clob.host.trim_end_matches('/');
        let sdk_client = ClobSdkClient::new(host, ClobConfig::default())?
            .authentication_builder(&signer)
            .authenticate()
            .await
            .context("Failed to authenticate with CLOB API")?;

        // 解析 token_id. trade.market_id is actually the assetId in hex (with or without 0x)
        let token_id_str = trade.market_id.strip_prefix("0x").unwrap_or(&trade.market_id);
        let token_id = U256::from_str_radix(token_id_str, 16)
            .context("Failed to parse token_id from trade event")?;

        let side = match trade.side {
            crate::monitor::Side::Buy => SdkSide::Buy,
            crate::monitor::Side::Sell => SdkSide::Sell,
        };

        let tick_size = sdk_client.tick_size(token_id).await
            .context("Failed to fetch tick size")?
            .minimum_tick_size
            .as_decimal();
        let decimals = tick_size.scale();

        let price = trade.price.round_dp(decimals);
        let size = size.round_dp(2);

        // Directly use the rust_decimal::Decimal that the SDK expects
        let order_builder = sdk_client.limit_order()
            .token_id(token_id)
            .side(side)
            .price(price)
            .size(size)
            .order_type(SdkOrderType::GTC);

        let order = order_builder.build().await
            .context("Failed to build order")?;
        
        let signed_order = sdk_client.sign(&signer, order).await
            .context("Failed to sign order")?;
        
        let response = sdk_client.post_order(signed_order).await
            .context("Failed to submit order")?;
        
        Ok(response.order_id.to_string())
    }

    /// 模拟执行（用于测试）
    async fn simulate_execution(&self, _trade: &TradeEvent, size: Decimal) -> Result<Decimal> {
        // 模拟跟单利润
        let profit = size * dec!(0.05); // 5% 模拟利润
        info!("Simulated copy: size={}, profit={}", size, profit);
        Ok(profit)
    }

    fn calculate_copy_size(&self, original_size: Decimal) -> Decimal {
        let copy_ratio = Decimal::from_f64_retain(self.config.strategy.copy_ratio).unwrap_or(dec!(1.0));
        let size = original_size * copy_ratio;
        
        let min_size = Decimal::from_f64_retain(self.config.strategy.min_trade_size_usd).unwrap_or(dec!(5.0));
        let max_size = Decimal::from_f64_retain(self.config.strategy.max_trade_size_usd).unwrap_or(dec!(1000.0));
        
        size.clamp(min_size, max_size)
    }
}
