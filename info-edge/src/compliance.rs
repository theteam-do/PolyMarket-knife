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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nlp::Direction;
    use crate::signal::Signal;

    fn create_test_compliance() -> ComplianceChecker {
        let config = RiskConfig {
            max_daily_loss: 5000.0,
            legal_review_required: false,
        };
        ComplianceChecker::new(&config)
    }

    fn create_test_signal() -> Signal {
        Signal {
            market: "test_market".to_string(),
            direction: Direction::Yes,
            confidence: 0.85,
            expected_return: 0.50,
            news_title: "Test news".to_string(),
        }
    }

    #[test]
    fn test_check_passes() {
        let checker = create_test_compliance();
        let signal = create_test_signal();
        
        let result = checker.check(&signal);
        assert!(result.is_ok());
    }

    #[test]
    fn test_daily_loss_limit() {
        let mut checker = create_test_compliance();
        
        checker.update_pnl(-6000.0);
        let signal = create_test_signal();
        
        let result = checker.check(&signal);
        assert!(result.is_err());
    }

    #[test]
    fn test_legal_review_required() {
        let config = RiskConfig {
            max_daily_loss: 5000.0,
            legal_review_required: true,
        };
        let mut checker = ComplianceChecker::new(&config);
        
        let signal = create_test_signal();
        let result = checker.check(&signal);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Legal review"));
    }

    #[test]
    fn test_audit_log() {
        let checker = create_test_compliance();
        let signal = create_test_signal();
        
        // Should not panic
        checker.audit_log(&signal);
    }

    #[test]
    fn test_update_pnl() {
        let mut checker = create_test_compliance();
        
        checker.update_pnl(100.0);
        assert_eq!(checker.daily_pnl, 100.0);
        
        checker.update_pnl(-50.0);
        assert_eq!(checker.daily_pnl, 50.0);
    }

    #[test]
    fn test_reset_daily() {
        let mut checker = create_test_compliance();
        
        checker.update_pnl(100.0);
        checker.reset_daily();
        
        assert_eq!(checker.daily_pnl, 0.0);
    }
}
