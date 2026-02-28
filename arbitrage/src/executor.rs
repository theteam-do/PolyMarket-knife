//! 套利执行器 - 简化版本

use anyhow::Result;
use polymarket_client_sdk::types::Decimal;
use rust_decimal_macros::dec;
use tracing::{info, instrument};

use crate::config::Config;
use crate::detector::ArbOpportunity;

pub struct Executor {
    #[allow(dead_code)]
    config: Config,
}

impl Executor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    #[instrument(skip(self), fields(opp = %opp))]
    pub async fn execute(&self, opp: &ArbOpportunity) -> Result<Decimal> {
        match opp {
            ArbOpportunity::BuyAndMint {
                profit_per_share,
                max_shares,
                ..
            } => {
                info!(
                    "Executing BuyAndMint: shares={} profit/share={}",
                    max_shares, profit_per_share
                );

                // TODO: 实际执行套利逻辑
                // 1. 买入 Yes 代币
                // 2. 买入 No 代币
                // 3. 调用合约 mint
                // 4. 调用合约 redeem

                Ok(*max_shares * dec!(0.02))
            }
            ArbOpportunity::RedeemAndSell { .. } => {
                info!("Executing RedeemAndSell (not implemented)");
                Ok(dec!(0))
            }
        }
    }
}
