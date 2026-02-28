//! 套利执行器 - 简化版本

use anyhow::Result;
use rust_decimal::Decimal;
use tracing::{info, instrument};

use crate::config::Config;
use crate::detector::ArbOpportunity;

pub struct Executor {
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
                    "Executing BuyAndMint arbitrage: {} shares @ ${}/share profit",
                    max_shares, profit_per_share
                );
                // TODO: 实现套利逻辑
                Ok(*profit_per_share * *max_shares)
            }
            ArbOpportunity::RedeemAndSell {
                profit_per_share,
                max_shares,
                ..
            } => {
                info!(
                    "Executing RedeemAndSell arbitrage: {} shares @ ${}/share profit",
                    max_shares, profit_per_share
                );
                // TODO: 实现反向套利
                Ok(*profit_per_share * *max_shares)
            }
        }
    }
}
