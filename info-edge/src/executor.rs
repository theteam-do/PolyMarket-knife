//! 交易执行器 - 使用官方 SDK

use anyhow::{Context, Result};
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::clob::types::{Side as SdkSide, Amount};
use polymarket_client_sdk::types::{Decimal, U256};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use rust_decimal_macros::dec;
use std::str::FromStr;
use tracing::{info, instrument};

use crate::config::Config as AppConfig;
use crate::signal::Signal;
use crate::nlp::Direction;

pub struct Executor {
    client: Client,
    config: AppConfig,
}

impl Executor {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let private_key = std::env::var(PRIVATE_KEY_VAR)
            .context("Need POLYMARKET_PRIVATE_KEY environment variable")?;
        
        let signer = LocalSigner::from_str(&private_key)?
            .with_chain_id(Some(POLYGON));

        let client = Client::new(&config.clob.host, Config::default())?
            .authentication_builder(&signer)
            .authenticate()
            .await
            .context("Failed to authenticate")?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    #[instrument(skip(self), fields(signal = ?signal))]
    pub async fn execute(&self, signal: &Signal) -> Result<()> {
        let position = self.calculate_position(signal.confidence);
        
        info!(
            "Executing order: {} {} @ confidence {:.2}, position ${}",
            match signal.direction {
                Direction::Yes => "BUY",
                Direction::No => "SELL",
                Direction::Neutral => "HOLD",
            },
            signal.market,
            signal.confidence,
            position
        );

        // TODO: 将市场映射到 token_id
        // 目前只是示例，需要实际的市场 ID
        let token_id = U256::from_str("123456789")?;
        
        let side = match signal.direction {
            Direction::Yes => SdkSide::Buy,
            Direction::No => SdkSide::Sell,
            Direction::Neutral => return Ok(()),
        };

        // 使用官方 SDK 下单
        let order = self.client
            .limit_order()
            .token_id(token_id)
            .price(dec!(0.50))  // 示例价格
            .amount(Amount::usdc(position)?)
            .side(side)
            .build()
            .await?;

        // 需要 signer 来签名订单
        // 这里简化处理，实际需要获取 signer
        info!("Order built: {:?}", order);
        
        Ok(())
    }

    fn calculate_position(&self, confidence: f64) -> Decimal {
        let base = self.config.strategy.max_position_usd;
        let base_dec = Decimal::from_f64_retain(base).unwrap_or(dec!(1000));
        
        // 高置信度用大仓位，低置信度用小仓位
        if confidence >= self.config.strategy.confidence_threshold {
            base_dec
        } else {
            base_dec * dec!(0.3)
        }
    }
}
