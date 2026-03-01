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

    async fn attack_low_gas(&self, target: &TargetMarket) -> Result<()> {
        if self.config.strategy.attack_gas_limit == 0 {
            anyhow::bail!("attack_gas_limit must be > 0");
        }

        // 参数校验
        if self.config.strategy.attack_gas_limit > 1000000 {
            anyhow::bail!("attack_gas_limit too high: {}", self.config.strategy.attack_gas_limit);
        }

        tracing::info!(
            "[SIMULATION] Low-gas attack prepared for market {}: gas_limit={}, timestamp={}",
            target.market,
            self.config.strategy.attack_gas_limit,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        // 模拟攻击准备时间
        let prep_time = std::time::Duration::from_millis(50 + (self.config.strategy.attack_gas_limit % 100));
        tokio::time::sleep(prep_time).await;
        
        tracing::debug!("Low-gas attack simulation completed for market {}", target.market);
        Ok(())
    }

    async fn attack_nonce_gap(&self, target: &TargetMarket) -> Result<()> {
        // 参数校验
        if !self.config.strategy.attack_nonce_gap {
            anyhow::bail!("nonce_gap attack is disabled in config");
        }

        tracing::info!(
            "[SIMULATION] Nonce-gap attack prepared for market {}: strategy=enabled, timestamp={}",
            target.market,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        // 模拟 nonce 间隙序列生成
        let gap_sequence = vec![1001, 1003, 1005]; // 模拟跳过的 nonce
        tracing::debug!("Nonce gap sequence: {:?}", gap_sequence);
        
        // 模拟攻击执行时间
        let exec_time = std::time::Duration::from_millis(50 + (gap_sequence.len() as u64 * 10));
        tokio::time::sleep(exec_time).await;
        
        tracing::debug!("Nonce-gap attack simulation completed for market {}", target.market);
        Ok(())
    }

    async fn attack_insufficient_balance(&self, target: &TargetMarket) -> Result<()> {
        // 参数校验
        if self.config.strategy.min_liquidity_usd <= 0.0 {
            anyhow::bail!("min_liquidity_usd must be > 0");
        }

        tracing::info!(
            "[SIMULATION] Insufficient-balance attack prepared for market {}: min_liquidity=${:.2}, timestamp={}",
            target.market,
            self.config.strategy.min_liquidity_usd,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        // 模拟余额不足场景
        let simulated_balance = self.config.strategy.min_liquidity_usd * 0.8; // 80% 的流动性
        tracing::debug!("Simulated balance: ${:.2} (80% of target liquidity)", simulated_balance);
        
        // 模拟攻击检测时间
        let detection_time = std::time::Duration::from_millis(50 + (simulated_balance as u64 % 100));
        tokio::time::sleep(detection_time).await;
        
        tracing::debug!("Insufficient-balance attack simulation completed for market {}", target.market);
        Ok(())
    }
}
