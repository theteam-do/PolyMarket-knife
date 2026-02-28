//! 合规检查器 ⚠️

use crate::config::RiskConfig;
use crate::signal::Signal;
use anyhow::Result;

pub struct ComplianceChecker {
    config: RiskConfig,
    daily_pnl: f64,
    trade_log: Vec<TradeRecord>,
}

struct TradeRecord {
    timestamp: u64,
    signal: String,
}

impl ComplianceChecker {
    pub fn new(config: &RiskConfig) -> Self {
        Self {
            config: config.clone(),
            daily_pnl: 0.0,
            trade_log: Vec::new(),
        }
    }

    pub fn check(&self, signal: &Signal) -> Result<()> {
        // 检查日亏损
        if self.daily_pnl < -self.config.max_daily_loss {
            return Err(anyhow::anyhow!("Daily loss limit reached"));
        }

        // ⚠️ 如果需要法律审查
        if self.config.legal_review_required {
            return Err(anyhow::anyhow!(
                "⚠️ Legal review required before trading. This signal requires manual approval."
            ));
        }

        // 记录审计日志
        self.audit_log(signal);

        Ok(())
    }

    fn audit_log(&self, signal: &Signal) {
        // TODO: 写入审计日志
        // 保留所有交易决策记录以备合规审查
        tracing::info!(
            target: "compliance_audit",
            "Signal: market={}, direction={:?}, confidence={:.2}, title={}",
            signal.market,
            signal.direction,
            signal.confidence,
            signal.news_title
        );
    }

    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
        self.trade_log.clear();
    }
}
