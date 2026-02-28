//! 攻击执行器
//!
//! ⚠️ 以下方法仅供学习理解攻击原理 ⚠️

use anyhow::Result;
use tracing::instrument;

use crate::config::Config;
use crate::scanner::TargetMarket;

pub struct AttackExecutor {
    config: Config,
}

impl AttackExecutor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    #[instrument(skip(self), fields(target = %target.market))]
    pub async fn execute(&self, target: &TargetMarket) -> Result<()> {
        // ⚠️ 以下方法仅供学习理解攻击原理 ⚠️
        // 实际实现需要深入理解 Polymarket 撮合机制

        // 方法 1: Gas 不足攻击
        // 发送 Gas 不足的交易，使其在链上失败
        // 这会导致匹配的订单被移除
        if let Err(e) = self.attack_low_gas(target).await {
            tracing::warn!("Low gas attack failed: {}", e);
        }

        // 方法 2: Nonce 间隙攻击
        // 制造 nonce 间隙，使交易顺序错乱
        if self.config.strategy.attack_nonce_gap {
            if let Err(e) = self.attack_nonce_gap(target).await {
                tracing::warn!("Nonce gap attack failed: {}", e);
            }
        }

        // 方法 3: 余额不足攻击
        // 用不足余额发起交易，使其失败
        if let Err(e) = self.attack_insufficient_balance(target).await {
            tracing::warn!("Insufficient balance attack failed: {}", e);
        }

        Ok(())
    }

    async fn attack_low_gas(&self, _target: &TargetMarket) -> Result<()> {
        if self.config.strategy.attack_gas_limit == 0 {
            anyhow::bail!("attack_gas_limit must be > 0");
        }

        tracing::info!(
            "[SIMULATION] low-gas scenario prepared: gas_limit={}",
            self.config.strategy.attack_gas_limit
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(())
    }

    async fn attack_nonce_gap(&self, _target: &TargetMarket) -> Result<()> {
        tracing::info!("[SIMULATION] nonce-gap sequence prepared");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(())
    }

    async fn attack_insufficient_balance(&self, _target: &TargetMarket) -> Result<()> {
        tracing::info!("[SIMULATION] insufficient-balance scenario prepared");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(())
    }
}
