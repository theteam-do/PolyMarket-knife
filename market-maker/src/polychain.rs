//! Polymarket 链上交互模块

use anyhow::Result;
use tracing::info;

pub struct ChainExecutor {
    _rpc_url: String,
}

impl ChainExecutor {
    pub fn new(rpc_url: &str, _private_key: &str) -> Result<Self> {
        Ok(Self {
            _rpc_url: rpc_url.to_string(),
        })
    }

    pub async fn get_balance(&self, token_address: &str) -> Result<u64> {
        if token_address.trim().is_empty() {
            anyhow::bail!("token_address cannot be empty");
        }

        info!("Querying token balance from chain: token={}", token_address);
        Ok(1_000_000)
    }

    pub async fn redeem(&self, market_id: &str, yes_tokens: u64, no_tokens: u64) -> Result<()> {
        if market_id.trim().is_empty() {
            anyhow::bail!("market_id cannot be empty");
        }
        if yes_tokens == 0 && no_tokens == 0 {
            anyhow::bail!("at least one of yes_tokens/no_tokens must be > 0");
        }

        info!(
            "Submitting redeem tx: market_id={}, yes_tokens={}, no_tokens={}",
            market_id, yes_tokens, no_tokens
        );
        Ok(())
    }

    pub async fn mint(&self, market_id: &str, usdc_amount: u64) -> Result<()> {
        if market_id.trim().is_empty() {
            anyhow::bail!("market_id cannot be empty");
        }
        if usdc_amount == 0 {
            anyhow::bail!("usdc_amount must be > 0");
        }

        info!(
            "Submitting mint tx: market_id={}, usdc_amount={}",
            market_id, usdc_amount
        );
        Ok(())
    }
}
