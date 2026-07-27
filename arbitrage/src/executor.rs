//! 套利执行器

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;
use tracing::{info, instrument, warn};

use secrecy::ExposeSecret;
use alloy::primitives::ChainId;
use alloy::providers::ProviderBuilder;
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::clob::types::{OrderType as SdkOrderType, Side as SdkSide};
use polymarket_client_sdk::clob::{Client as ClobSdkClient, Config as ClobConfig};
use polymarket_client_sdk::ctf::types::{MergePositionsRequest, SplitPositionRequest};
use polymarket_client_sdk::ctf::Client as CtfClient;
use polymarket_client_sdk::types::U256;

use crate::config::Config;
use crate::config::ExecutionMode;
use crate::detector::ArbOpportunity;
use crate::quant::{build_runtime_quant_context, RuntimeQuantContext};
use crate::reporting::{ExecutionDisposition, ExecutionReport};
use crate::settlement::{SettlementDecision, SettlementPlan, SettlementResult};

const POLYGON_CHAIN_ID: ChainId = 137;

/// 套利执行参数
#[derive(Debug, Clone)]
struct ArbitrageParams {
    token_id_yes: String,
    token_id_no: String,
    price_yes: Decimal,
    price_no: Decimal,
    profit_per_share: Decimal,
    shares: Decimal,
}

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

    async fn execute_paper(&self, opp: &ArbOpportunity) -> Result<ExecutionReport> {
        let quant = build_runtime_quant_context(opp, &self.config.quant)?;
        let expected_edge = quant.expected_edge_usd();
        let settlement = self.simulated_settlement_for(opp, &quant)?;

        info!(
            "[PAPER] Arbitrage execution simulated: opportunity={} expected_edge={}",
            opp, expected_edge
        );
        Ok(
            ExecutionReport::new(ExecutionDisposition::PaperSimulated, expected_edge, None)
                .with_quant(quant.to_execution_quant())
                .with_settlement_option(settlement),
        )
    }

    fn settlement_decision(
        &self,
        opportunity: &ArbOpportunity,
        quant: &RuntimeQuantContext,
    ) -> Result<SettlementDecision> {
        SettlementPlan::build(opportunity, quant.effective_shares, &self.config.ctf)
    }

    fn simulated_settlement_for(
        &self,
        opportunity: &ArbOpportunity,
        quant: &RuntimeQuantContext,
    ) -> Result<Option<SettlementResult>> {
        Ok(match self.settlement_decision(opportunity, quant)? {
            SettlementDecision::Disabled => None,
            SettlementDecision::Skipped { action, reason } => {
                Some(SettlementResult::skipped(action, reason))
            }
            SettlementDecision::Ready(plan) => Some(SettlementResult::simulated(&plan)),
        })
    }

    fn fallback_settlement_for(&self, decision: &SettlementDecision) -> Option<SettlementResult> {
        match decision {
            SettlementDecision::Disabled => None,
            SettlementDecision::Skipped { action, reason } => {
                Some(SettlementResult::skipped(*action, reason.clone()))
            }
            SettlementDecision::Ready(plan) => Some(SettlementResult::simulated(plan)),
        }
    }

    async fn execute_settlement_plan(&self, plan: &SettlementPlan) -> Result<SettlementResult> {
        let private_key = self.config.polygon.private_key.expose_secret().trim();
        if private_key.is_empty() {
            anyhow::bail!("Private key not configured for CTF settlement");
        }

        let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
        let signer = LocalSigner::from_str(pk)
            .context("Failed to parse private key for CTF settlement")?
            .with_chain_id(Some(POLYGON_CHAIN_ID));

        let provider = ProviderBuilder::new()
            .wallet(signer)
            .connect(self.config.polygon.rpc_url.as_str())
            .await
            .context("Failed to connect provider for CTF settlement")?;
        let client = CtfClient::new(provider, POLYGON_CHAIN_ID)
            .context("Failed to initialize CTF client")?;

        match plan.action {
            crate::settlement::SettlementAction::MergePositions => {
                let request = MergePositionsRequest::for_binary_market(
                    plan.collateral_token,
                    plan.condition_id,
                    plan.collateral_amount,
                );
                let response = client
                    .merge_positions(&request)
                    .await
                    .context("Failed to merge positions")?;
                Ok(SettlementResult::confirmed(
                    plan,
                    response.transaction_hash.to_string(),
                    response.block_number,
                ))
            }
            crate::settlement::SettlementAction::SplitPosition => {
                let request = SplitPositionRequest::for_binary_market(
                    plan.collateral_token,
                    plan.condition_id,
                    plan.collateral_amount,
                );
                let response = client
                    .split_position(&request)
                    .await
                    .context("Failed to split position")?;
                Ok(SettlementResult::confirmed(
                    plan,
                    response.transaction_hash.to_string(),
                    response.block_number,
                ))
            }
        }
    }

    #[instrument(skip(self), fields(opp = %opp))]
    pub async fn execute(&self, opp: &ArbOpportunity) -> Result<ExecutionReport> {
        if self.config.execution.mode == ExecutionMode::Paper {
            return self.execute_paper(opp).await;
        }

        let quant = build_runtime_quant_context(opp, &self.config.quant)?;

        match opp {
            ArbOpportunity::BuyAndMint {
                token_id_yes,
                token_id_no,
                price_yes,
                price_no,
                profit_per_share,
                max_shares: _,
                market_id,
                ..
            } => {
                let params = ArbitrageParams {
                    token_id_yes: token_id_yes.clone(),
                    token_id_no: token_id_no.clone(),
                    price_yes: *price_yes,
                    price_no: *price_no,
                    profit_per_share: *profit_per_share,
                    shares: quant.effective_shares,
                };

                info!(
                    "Executing BuyAndMint: market={}, yes_token={}, no_token={}, shares={}, expected_edge/share={}",
                    market_id, params.token_id_yes, params.token_id_no, params.shares, params.profit_per_share
                );

                let settlement_decision = self.settlement_decision(opp, &quant)?;
                self.execute_buy_and_mint(market_id, &params, &quant, &settlement_decision)
                    .await
            }
            ArbOpportunity::RedeemAndSell {
                token_id_yes,
                token_id_no,
                price_yes,
                price_no,
                profit_per_share,
                max_shares: _,
                market_id,
                ..
            } => {
                let params = ArbitrageParams {
                    token_id_yes: token_id_yes.clone(),
                    token_id_no: token_id_no.clone(),
                    price_yes: *price_yes,
                    price_no: *price_no,
                    profit_per_share: *profit_per_share,
                    shares: quant.effective_shares,
                };

                info!(
                    "Executing RedeemAndSell: market={}, yes_token={}, no_token={}, shares={}, expected_edge/share={}",
                    market_id, params.token_id_yes, params.token_id_no, params.shares, params.profit_per_share
                );

                let settlement_decision = self.settlement_decision(opp, &quant)?;
                self.execute_redeem_and_sell(market_id, &params, &quant, &settlement_decision)
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
        params: &ArbitrageParams,
        quant: &RuntimeQuantContext,
        settlement_decision: &SettlementDecision,
    ) -> Result<ExecutionReport> {
        if params.shares <= Decimal::ZERO || params.profit_per_share <= Decimal::ZERO {
            warn!(
                "Skip buy-and-mint due to invalid params: shares={}, edge/share={}",
                params.shares, params.profit_per_share
            );
            return Ok(ExecutionReport::new(
                ExecutionDisposition::PaperSimulated,
                quant.expected_edge_usd(),
                None,
            )
            .with_quant(quant.to_execution_quant()));
        }

        let total_cost = (dec!(1.0) - params.profit_per_share) * params.shares;
        let expected_edge = quant.expected_edge_usd();
        let settlement_plan = match settlement_decision {
            SettlementDecision::Disabled => None,
            SettlementDecision::Skipped { action, reason } => {
                if self.config.execution.live_failure_fallback_to_paper {
                    return Ok(ExecutionReport::new(
                        ExecutionDisposition::LiveFallbackToPaper,
                        expected_edge,
                        None,
                    )
                    .with_quant(quant.to_execution_quant())
                    .with_settlement(SettlementResult::skipped(*action, reason.clone())));
                }
                anyhow::bail!("CTF settlement unavailable for buy-and-mint: {reason}");
            }
            SettlementDecision::Ready(plan) => Some(plan.clone()),
        };

        if let Err(e) = self
            .place_clob_orders(
                &params.token_id_yes,
                &params.token_id_no,
                params.price_yes,
                params.price_no,
                params.shares,
                SdkSide::Buy,
            )
            .await
        {
            if self.config.execution.live_failure_fallback_to_paper {
                warn!(
                    "Live submit failed, fallback to paper mode for buy-and-mint: {:?}",
                    e
                );
                return Ok(ExecutionReport::new(
                    ExecutionDisposition::LiveFallbackToPaper,
                    expected_edge,
                    None,
                )
                .with_quant(quant.to_execution_quant())
                .with_settlement_option(self.fallback_settlement_for(settlement_decision)));
            }
            return Err(e);
        }

        let settlement = if let Some(plan) = settlement_plan.as_ref() {
            match self.execute_settlement_plan(plan).await {
                Ok(result) => Some(result),
                Err(error) => {
                    warn!("Post-trade merge settlement failed: {}", error);
                    Some(SettlementResult::failed(plan, error.to_string()))
                }
            }
        } else {
            None
        };

        info!(
            "BuyAndMint (CLOB orders) submitted: shares={}, cost={}, expected_edge={}",
            params.shares, total_cost, expected_edge
        );

        Ok(
            ExecutionReport::new(ExecutionDisposition::LiveSubmitted, expected_edge, None)
                .with_quant(quant.to_execution_quant())
                .with_settlement_option(settlement),
        )
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
        params: &ArbitrageParams,
        quant: &RuntimeQuantContext,
        settlement_decision: &SettlementDecision,
    ) -> Result<ExecutionReport> {
        if params.shares <= Decimal::ZERO || params.profit_per_share <= Decimal::ZERO {
            warn!(
                "Skip redeem-and-sell due to invalid params: shares={}, edge/share={}",
                params.shares, params.profit_per_share
            );
            return Ok(ExecutionReport::new(
                ExecutionDisposition::PaperSimulated,
                quant.expected_edge_usd(),
                None,
            )
            .with_quant(quant.to_execution_quant()));
        }

        let expected_edge = quant.expected_edge_usd();
        let settlement = match settlement_decision {
            SettlementDecision::Disabled => None,
            SettlementDecision::Skipped { action, reason } => {
                if self.config.execution.live_failure_fallback_to_paper {
                    return Ok(ExecutionReport::new(
                        ExecutionDisposition::LiveFallbackToPaper,
                        expected_edge,
                        None,
                    )
                    .with_quant(quant.to_execution_quant())
                    .with_settlement(SettlementResult::skipped(*action, reason.clone())));
                }
                anyhow::bail!("CTF settlement unavailable for redeem-and-sell: {reason}");
            }
            SettlementDecision::Ready(plan) => match self.execute_settlement_plan(plan).await {
                Ok(result) => Some(result),
                Err(error) => {
                    if self.config.execution.live_failure_fallback_to_paper {
                        return Ok(ExecutionReport::new(
                            ExecutionDisposition::LiveFallbackToPaper,
                            expected_edge,
                            None,
                        )
                        .with_quant(quant.to_execution_quant())
                        .with_settlement(SettlementResult::failed(plan, error.to_string())));
                    }
                    return Err(error).context("Split settlement failed before order submit");
                }
            },
        };

        if let Err(e) = self
            .place_clob_orders(
                &params.token_id_yes,
                &params.token_id_no,
                params.price_yes,
                params.price_no,
                params.shares,
                SdkSide::Sell,
            )
            .await
        {
            if settlement.is_some() {
                return Err(e)
                    .context("CLOB order submission failed after successful split settlement");
            }
            if self.config.execution.live_failure_fallback_to_paper {
                warn!(
                    "Live submit failed, fallback to paper mode for redeem-and-sell: {:?}",
                    e
                );
                return Ok(ExecutionReport::new(
                    ExecutionDisposition::LiveFallbackToPaper,
                    expected_edge,
                    None,
                )
                .with_quant(quant.to_execution_quant())
                .with_settlement_option(self.fallback_settlement_for(settlement_decision)));
            }
            return Err(e);
        }

        info!(
            "RedeemAndSell (CLOB orders) submitted: shares={}, expected_edge={}",
            params.shares, expected_edge
        );
        Ok(
            ExecutionReport::new(ExecutionDisposition::LiveSubmitted, expected_edge, None)
                .with_quant(quant.to_execution_quant())
                .with_settlement_option(settlement),
        )
    }

    fn credentials(&self) -> Result<Option<Credentials>> {
        match (
            self.config
                .clob
                .api_key
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
            self.config
                .clob
                .api_secret
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
            self.config
                .clob
                .passphrase
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
        ) {
            (Some(api_key), Some(api_secret), Some(passphrase)) => Ok(Some(Credentials::new(
                uuid::Uuid::parse_str(api_key).context("invalid clob.api_key UUID")?,
                api_secret.to_string(),
                passphrase.to_string(),
            ))),
            _ => Ok(None),
        }
    }

    async fn place_clob_orders(
        &self,
        token_id_yes: &str,
        token_id_no: &str,
        price_yes: Decimal,
        price_no: Decimal,
        size: Decimal,
        side: SdkSide,
    ) -> Result<()> {
        let private_key = self.config.polygon.private_key.expose_secret().trim();
        if private_key.is_empty() {
            anyhow::bail!("Private key not configured for live execution");
        }

        let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
        let signer = LocalSigner::from_str(pk)
            .context("Failed to parse private key")?
            .with_chain_id(Some(POLYGON_CHAIN_ID));

        let host = self.config.clob.host.trim_end_matches('/');
        let sdk_config = if let Some(proxy_url) = &self.config.clob.proxy_url {
            ClobConfig::builder().proxy_url(proxy_url.clone()).build()
        } else {
            ClobConfig::default()
        };
        let mut auth = ClobSdkClient::new(host, sdk_config)?.authentication_builder(&signer);
        if let Some(credentials) = self.credentials()? {
            auth = auth.credentials(credentials);
        }
        let sdk_client = auth
            .authenticate()
            .await
            .context("Failed to authenticate with CLOB API")?;

        let token_yes_u256 =
            U256::from_str_radix(token_id_yes, 10).context("Failed to parse yes_token_id")?;
        let token_no_u256 =
            U256::from_str_radix(token_id_no, 10).context("Failed to parse no_token_id")?;

        let tick_size_yes = sdk_client
            .tick_size(token_yes_u256)
            .await
            .context("Failed to fetch yes tick size")?
            .minimum_tick_size
            .as_decimal();
        let tick_size_no = sdk_client
            .tick_size(token_no_u256)
            .await
            .context("Failed to fetch no tick size")?
            .minimum_tick_size
            .as_decimal();

        let price_yes = price_yes.round_dp(tick_size_yes.scale());
        let price_no = price_no.round_dp(tick_size_no.scale());
        let size = size.round_dp(2);

        // 1. Order YES
        let order_yes_builder = sdk_client
            .limit_order()
            .token_id(token_yes_u256)
            .side(side)
            .price(price_yes)
            .size(size)
            .order_type(SdkOrderType::FAK); // Use Fill And Kill (IOC) for arbitrage legs to avoid hanging orders

        let order_yes = match order_yes_builder.build().await {
            Ok(o) => o,
            Err(e) => {
                tracing::error!("Failed to build yes order: {:?}", e);
                return Err(anyhow::anyhow!("Failed to build yes order"));
            }
        };
        let signed_order_yes = sdk_client
            .sign(&signer, order_yes)
            .await
            .context("Failed to sign yes order")?;

        // 2. Order NO
        let order_no_builder = sdk_client
            .limit_order()
            .token_id(token_no_u256)
            .side(side)
            .price(price_no)
            .size(size)
            .order_type(SdkOrderType::FAK); // Use Fill And Kill (IOC)

        let order_no = match order_no_builder.build().await {
            Ok(o) => o,
            Err(e) => {
                tracing::error!("Failed to build no order: {:?}", e);
                return Err(anyhow::anyhow!("Failed to build no order"));
            }
        };
        let signed_order_no = sdk_client
            .sign(&signer, order_no)
            .await
            .context("Failed to sign no order")?;

        // Ideally, we could use batch order submission if the SDK supported it.
        // We will send them sequentially.
        let resp_yes = sdk_client
            .post_order(signed_order_yes)
            .await
            .context("Failed to post yes order")?;
        info!("Yes leg order posted: {}", resp_yes.order_id);

        let resp_no = sdk_client
            .post_order(signed_order_no)
            .await
            .context("Failed to post no order")?;
        info!("No leg order posted: {}", resp_no.order_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::config::QuantConfig;
    use crate::detector::ArbOpportunity;
    use crate::quant::build_runtime_quant_context;

    #[test]
    fn runtime_quant_computes_edges_and_kelly_target() {
        let opportunity = sample_opportunity();
        let quant = build_runtime_quant_context(
            &opportunity,
            &QuantConfig {
                fees_bps: 100.0,
                slippage_bps: 50.0,
                latency_penalty_bps: 25.0,
                rebate_bps: 10.0,
                gas_usd_override: Some(2.0),
                fill_probability_override: Some(0.8),
                posterior_prob_override: Some(0.70),
                net_odds: 1.0,
                fraction_of_kelly: 0.50,
                bankroll_usd: Some(1000.0),
                max_notional_usd: Some(40.0),
                apply_kelly_sizing: true,
                ..Default::default()
            },
        )
        .expect("runtime quant should build");

        assert_eq!(quant.implied_prob, Some(dec!(0.50)));
        assert_eq!(quant.posterior_prob, Some(dec!(0.70)));
        assert_eq!(quant.gross_edge_usd, dec!(40.0));
        assert_eq!(quant.fees_usd.round_dp(2), dec!(0.40));
        assert_eq!(quant.slippage_usd.round_dp(2), dec!(0.20));
        assert_eq!(quant.latency_penalty_usd.round_dp(2), dec!(0.10));
        assert_eq!(quant.gas_usd.round_dp(2), dec!(2.00));
        assert_eq!(quant.rebate_usd.round_dp(2), dec!(0.04));
        assert_eq!(quant.net_edge_usd.round_dp(2), dec!(37.34));
        assert_eq!(quant.fill_probability, Some(dec!(0.80)));
        assert_eq!(
            quant.expected_net_edge_after_fill_usd.round_dp(2),
            dec!(29.87)
        );
        assert_eq!(quant.target_fraction, Some(dec!(0.20)));
        assert_eq!(quant.effective_shares.round_dp(0), dec!(80));
    }

    #[test]
    fn runtime_quant_uses_runtime_probability_source_for_kelly_target() {
        let opportunity = sample_opportunity();
        let quant = build_runtime_quant_context(
            &opportunity,
            &QuantConfig {
                net_odds: 1.0,
                fraction_of_kelly: 0.50,
                bankroll_usd: Some(1000.0),
                max_notional_usd: Some(40.0),
                apply_kelly_sizing: true,
                ..Default::default()
            },
        )
        .expect("runtime quant should build");

        assert_eq!(quant.implied_prob, Some(dec!(0.50)));
        assert!(
            quant
                .posterior_prob
                .expect("runtime source should set posterior")
                > dec!(0.50)
        );
        assert!(
            quant
                .target_fraction
                .expect("runtime source should feed Kelly")
                > dec!(0.0)
        );
        assert!(quant.effective_shares < dec!(100));
    }

    #[test]
    fn runtime_quant_keeps_full_size_when_kelly_is_not_applied() {
        let opportunity = sample_opportunity();
        let quant = build_runtime_quant_context(&opportunity, &QuantConfig::default())
            .expect("runtime quant should build");

        assert_eq!(quant.implied_prob, Some(dec!(0.50)));
        assert!(quant.posterior_prob.is_some());
        assert_eq!(quant.gross_edge_usd, dec!(50.0));
        assert_eq!(quant.net_edge_usd, dec!(50.0));
        assert_eq!(quant.expected_net_edge_after_fill_usd, dec!(50.0));
        assert!(quant.target_fraction.is_some());
        assert_eq!(quant.effective_shares, dec!(100));
    }

    fn sample_opportunity() -> ArbOpportunity {
        ArbOpportunity::BuyAndMint {
            market_id: "fixture-market".to_string(),
            condition_id: Some(
                "0x0000000000000000000000000000000000000000000000000000000000000042".to_string(),
            ),
            token_id_yes: "101".to_string(),
            token_id_no: "202".to_string(),
            price_yes: dec!(0.25),
            price_no: dec!(0.25),
            profit_per_share: dec!(0.50),
            max_shares: dec!(100),
        }
    }
}
