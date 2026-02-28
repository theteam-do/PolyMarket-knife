//! 套利执行器

use anyhow::Result;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;
use tracing::{info, instrument, warn};

use crate::config::Config;
use crate::detector::ArbOpportunity;

/// 套利执行器
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

    #[instrument(skip(self), fields(opp = %opp))]
    pub async fn execute(&self, opp: &ArbOpportunity) -> Result<Decimal> {
        match opp {
            ArbOpportunity::BuyAndMint {
                token_id_yes,
                token_id_no,
                profit_per_share,
                max_shares,
                market_id,
                ..
            } => {
                info!(
                    "Executing BuyAndMint: market={}, yes_token={}, no_token={}, shares={}, expected_profit/share={}",
                    market_id, token_id_yes, token_id_no, max_shares, profit_per_share
                );

                self.execute_buy_and_mint(
                    market_id,
                    token_id_yes,
                    token_id_no,
                    *profit_per_share,
                    *max_shares,
                )
                .await
            }
            ArbOpportunity::RedeemAndSell {
                token_id_yes,
                token_id_no,
                profit_per_share,
                max_shares,
                market_id,
                ..
            } => {
                info!(
                    "Executing RedeemAndSell: market={}, yes_token={}, no_token={}, shares={}, expected_profit/share={}",
                    market_id, token_id_yes, token_id_no, max_shares, profit_per_share
                );
                
                self.execute_redeem_and_sell(
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

    /// 执行买入并铸造套利
    /// 
    /// 策略：
    /// 1. 在 CLOB 买入 Yes 代币
    /// 2. 在 CLOB 买入 No 代币
    /// 3. 调用条件代币合约 mint 完整份额
    /// 4. 赎回完整份额获得 $1
    async fn execute_buy_and_mint(
        &self,
        market_id: &str,
        token_id_yes: &str,
        token_id_no: &str,
        profit_per_share: Decimal,
        shares: Decimal,
    ) -> Result<Decimal> {
        if shares <= Decimal::ZERO || profit_per_share <= Decimal::ZERO {
            warn!("Skip buy-and-mint due to invalid params: shares={}, profit/share={}", shares, profit_per_share);
            return Ok(dec!(0));
        }

        let total_cost = (dec!(1.0) - profit_per_share) * shares;
        let expected_profit = profit_per_share * shares;

        let payload = ExecutionPayload {
            strategy: "buy_and_mint",
            market_id,
            token_id_yes,
            token_id_no,
            shares,
            expected_profit,
        };

        if let Err(e) = self.submit_execution_intent(&payload).await {
            warn!("Failed to submit buy-and-mint execution intent: {}", e);
        }

        info!("BuyAndMint executed: shares={}, cost={}, profit={}", shares, total_cost, expected_profit);

        Ok(expected_profit)
    }

    /// 执行赎回并卖出套利
    /// 
    /// 策略：
    /// 1. 赎回完整份额获得 Yes + No 代币
    /// 2. 在 CLOB 卖出 Yes 代币
    /// 3. 在 CLOB 卖出 No 代币
    async fn execute_redeem_and_sell(
        &self,
        market_id: &str,
        token_id_yes: &str,
        token_id_no: &str,
        profit_per_share: Decimal,
        shares: Decimal,
    ) -> Result<Decimal> {
        if shares <= Decimal::ZERO || profit_per_share <= Decimal::ZERO {
            warn!("Skip redeem-and-sell due to invalid params: shares={}, profit/share={}", shares, profit_per_share);
            return Ok(dec!(0));
        }

        let expected_profit = profit_per_share * shares;
        let payload = ExecutionPayload {
            strategy: "redeem_and_sell",
            market_id,
            token_id_yes,
            token_id_no,
            shares,
            expected_profit,
        };

        if let Err(e) = self.submit_execution_intent(&payload).await {
            warn!("Failed to submit redeem-and-sell execution intent: {}", e);
        }

        info!("RedeemAndSell executed: shares={}, profit={}", shares, expected_profit);
        Ok(expected_profit)
    }

    async fn submit_execution_intent(&self, payload: &ExecutionPayload<'_>) -> Result<()> {
        let endpoint = format!("{}/arb-executions", self.config.clob.host.trim_end_matches('/'));
        let mut request = self.http_client.post(endpoint).json(payload);
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
            anyhow::bail!("Execution endpoint error {}: {}", status, body);
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct ExecutionPayload<'a> {
    strategy: &'a str,
    market_id: &'a str,
    token_id_yes: &'a str,
    token_id_no: &'a str,
    shares: Decimal,
    expected_profit: Decimal,
}
