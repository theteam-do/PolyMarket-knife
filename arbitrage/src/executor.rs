//! 套利执行器

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;
use tracing::{info, instrument, warn};

use polymarket_client_sdk::clob::{Client as ClobSdkClient, Config as ClobConfig};
use polymarket_client_sdk::clob::types::{Side as SdkSide, OrderType as SdkOrderType};
use polymarket_client_sdk::types::U256;
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use alloy::primitives::ChainId;

use crate::config::Config;
use crate::config::ExecutionMode;
use crate::detector::ArbOpportunity;

const POLYGON_CHAIN_ID: ChainId = 137;

/// 套利执行器
pub struct Executor {
    config: Config,
}

impl Executor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    async fn execute_paper(&self, opp: &ArbOpportunity) -> Result<Decimal> {
        let profit = match opp {
            ArbOpportunity::BuyAndMint {
                profit_per_share,
                max_shares,
                ..
            }
            | ArbOpportunity::RedeemAndSell {
                profit_per_share,
                max_shares,
                ..
            } => *profit_per_share * *max_shares,
        };

        info!(
            "[PAPER] Arbitrage execution simulated: opportunity={} expected_profit={}",
            opp, profit
        );
        Ok(profit)
    }

    #[instrument(skip(self), fields(opp = %opp))]
    pub async fn execute(&self, opp: &ArbOpportunity) -> Result<Decimal> {
        if self.config.execution.mode == ExecutionMode::Paper {
            return self.execute_paper(opp).await;
        }

        match opp {
            ArbOpportunity::BuyAndMint {
                token_id_yes,
                token_id_no,
                price_yes,
                price_no,
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
                    *price_yes,
                    *price_no,
                    *profit_per_share,
                    *max_shares,
                )
                .await
            }
            ArbOpportunity::RedeemAndSell {
                token_id_yes,
                token_id_no,
                price_yes,
                price_no,
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
                    *price_yes,
                    *price_no,
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
    /// 3. （可选后续步骤：调用条件代币合约 mint/merge 完整份额）
    async fn execute_buy_and_mint(
        &self,
        _market_id: &str,
        token_id_yes: &str,
        token_id_no: &str,
        price_yes: Decimal,
        price_no: Decimal,
        profit_per_share: Decimal,
        shares: Decimal,
    ) -> Result<Decimal> {
        if shares <= Decimal::ZERO || profit_per_share <= Decimal::ZERO {
            warn!("Skip buy-and-mint due to invalid params: shares={}, profit/share={}", shares, profit_per_share);
            return Ok(dec!(0));
        }

        let total_cost = (dec!(1.0) - profit_per_share) * shares;
        let expected_profit = profit_per_share * shares;

        if let Err(e) = self.place_clob_orders(token_id_yes, token_id_no, price_yes, price_no, shares, SdkSide::Buy).await {
            if self.config.execution.live_failure_fallback_to_paper {
                warn!(
                    "Live submit failed, fallback to paper mode for buy-and-mint: {:?}", 
                    e
                );
                return Ok(expected_profit);
            }
            return Err(e);
        }

        info!("BuyAndMint (CLOB orders) executed: shares={}, cost={}, expected_profit={}", shares, total_cost, expected_profit);

        Ok(expected_profit)
    }

    /// 执行赎回并卖出套利
    /// 
    /// 策略：
    /// 1. （可选前置步骤：调用条件代币合约 split 份额）
    /// 2. 在 CLOB 卖出 Yes 代币
    /// 3. 在 CLOB 卖出 No 代币
    async fn execute_redeem_and_sell(
        &self,
        _market_id: &str,
        token_id_yes: &str,
        token_id_no: &str,
        price_yes: Decimal,
        price_no: Decimal,
        profit_per_share: Decimal,
        shares: Decimal,
    ) -> Result<Decimal> {
        if shares <= Decimal::ZERO || profit_per_share <= Decimal::ZERO {
            warn!("Skip redeem-and-sell due to invalid params: shares={}, profit/share={}", shares, profit_per_share);
            return Ok(dec!(0));
        }

        let expected_profit = profit_per_share * shares;

        if let Err(e) = self.place_clob_orders(token_id_yes, token_id_no, price_yes, price_no, shares, SdkSide::Sell).await {
            if self.config.execution.live_failure_fallback_to_paper {
                warn!(
                    "Live submit failed, fallback to paper mode for redeem-and-sell: {:?}", 
                    e
                );
                return Ok(expected_profit);
            }
            return Err(e);
        }

        info!("RedeemAndSell (CLOB orders) executed: shares={}, profit={}", shares, expected_profit);
        Ok(expected_profit)
    }

    async fn place_clob_orders(&self, token_id_yes: &str, token_id_no: &str, price_yes: Decimal, price_no: Decimal, size: Decimal, side: SdkSide) -> Result<()> {
        let private_key = self.config.clob.api_secret.as_deref()
            .or(Some(&self.config.polygon.private_key))
            .context("Private key not configured for live execution")?;
        
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

        let token_yes_u256 = U256::from_str_radix(token_id_yes, 10)
            .context("Failed to parse yes_token_id")?;
        let token_no_u256 = U256::from_str_radix(token_id_no, 10)
            .context("Failed to parse no_token_id")?;

        let tick_size_yes = sdk_client.tick_size(token_yes_u256).await
            .context("Failed to fetch yes tick size")?
            .minimum_tick_size
            .as_decimal();
        let tick_size_no = sdk_client.tick_size(token_no_u256).await
            .context("Failed to fetch no tick size")?
            .minimum_tick_size
            .as_decimal();

        let price_yes = price_yes.round_dp(tick_size_yes.scale());
        let price_no = price_no.round_dp(tick_size_no.scale());
        let size = size.round_dp(2);

        // 1. Order YES
        let order_yes_builder = sdk_client.limit_order()
            .token_id(token_yes_u256)
            .side(side)
            .price(price_yes)
            .size(size)
            .order_type(SdkOrderType::FAK); // Use Fill And Kill (IOC) for arbitrage legs to avoid hanging orders
        
        let order_yes = match order_yes_builder.build().await { Ok(o) => o, Err(e) => { tracing::error!("Failed to build yes order: {:?}", e); return Err(anyhow::anyhow!("Failed to build yes order")); } };
        let signed_order_yes = sdk_client.sign(&signer, order_yes).await.context("Failed to sign yes order")?;

        // 2. Order NO
        let order_no_builder = sdk_client.limit_order()
            .token_id(token_no_u256)
            .side(side)
            .price(price_no)
            .size(size)
            .order_type(SdkOrderType::FAK); // Use Fill And Kill (IOC)
            
        let order_no = match order_no_builder.build().await { Ok(o) => o, Err(e) => { tracing::error!("Failed to build no order: {:?}", e); return Err(anyhow::anyhow!("Failed to build no order")); } };
        let signed_order_no = sdk_client.sign(&signer, order_no).await.context("Failed to sign no order")?;

        // Ideally, we could use batch order submission if the SDK supported it.
        // We will send them sequentially.
        let resp_yes = sdk_client.post_order(signed_order_yes).await.context("Failed to post yes order")?;
        info!("Yes leg order posted: {}", resp_yes.order_id);

        let resp_no = sdk_client.post_order(signed_order_no).await.context("Failed to post no order")?;
        info!("No leg order posted: {}", resp_no.order_id);

        Ok(())
    }
}
