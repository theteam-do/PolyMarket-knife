//! 套利执行器 - 使用 poly-client

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use tracing::{info, instrument};

use crate::config::Config;
use crate::detector::ArbOpportunity;

pub struct Executor {
    client: PolyClient,
}

impl Executor {
    pub fn new(config: &Config) -> Self {
        let client = if config.polygon.private_key.is_empty() {
            PolyClient::new(&config.clob.host)
        } else {
            PolyClient::with_auth(&config.clob.host, &config.to_auth_config())
        };

        Self { client }
    }

    #[instrument(skip(self), fields(opp = %opp))]
    pub async fn execute(&self, opp: &ArbOpportunity) -> Result<Decimal> {
        match opp {
            ArbOpportunity::BuyAndMint {
                market_id,
                token_id_yes,
                token_id_no,
                profit_per_share,
                max_shares,
            } => {
                self.buy_and_mint(
                    market_id,
                    token_id_yes,
                    token_id_no,
                    *profit_per_share,
                    *max_shares,
                )
                .await
            }
            ArbOpportunity::RedeemAndSell {
                market_id,
                token_id_yes,
                token_id_no,
                profit_per_share,
                max_shares,
            } => {
                self.redeem_and_sell(
                    market_id,
                    token_id_yes,
                    token_id_no,
                    *profit_per_share,
                    *max_shares,
                )
                .await
            }
        }
    }

    #[instrument(skip(self))]
    async fn buy_and_mint(
        &self,
        _market_id: &str,
        token_id_yes: &str,
        token_id_no: &str,
        profit_per_share: Decimal,
        max_shares: Decimal,
    ) -> Result<Decimal> {
        info!(
            "Executing BuyAndMint arbitrage: {} shares @ ${}/share profit",
            max_shares, profit_per_share
        );

        // TODO: 在 CLOB 买入 Yes 和 No 代币
        // 1. 下买单买入 Yes
        // 2. 下买单买入 No
        // 3. 调用合约 mint
        // 4. 调用合约 redeem

        // 模拟执行
        let estimated_profit = profit_per_share * max_shares;
        
        Ok(estimated_profit)
    }

    #[instrument(skip(self))]
    async fn redeem_and_sell(
        &self,
        _market_id: &str,
        token_id_yes: &str,
        token_id_no: &str,
        profit_per_share: Decimal,
        max_shares: Decimal,
    ) -> Result<Decimal> {
        info!(
            "Executing RedeemAndSell arbitrage: {} shares @ ${}/share profit",
            max_shares, profit_per_share
        );

        // TODO: 实现反向套利（需要已有持仓）
        let estimated_profit = profit_per_share * max_shares;
        
        Ok(estimated_profit)
    }
}
