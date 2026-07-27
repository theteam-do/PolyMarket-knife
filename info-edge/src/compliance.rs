//! 合规检查器

use anyhow::Result;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

use crate::config::RiskConfig;
use crate::signal::Signal;

pub struct ComplianceChecker {
    config: RiskConfig,
    daily_pnl: Decimal,
}

impl ComplianceChecker {
    pub fn new(config: &RiskConfig) -> Self {
        Self {
            config: config.clone(),
            daily_pnl: dec!(0),
        }
    }

    pub fn check(&self, signal: &Signal) -> Result<()> {
        // 检查日亏损
        let max_loss = Decimal::from_f64_retain(self.config.max_daily_loss).unwrap();
        if self.daily_pnl < -max_loss {
            return Err(anyhow::anyhow!("Daily loss limit reached"));
        }

        // 如果需要法律审查
        if self.config.legal_review_required {
            return Err(anyhow::anyhow!("Legal review required before trading"));
        }

        // 记录审计日志
        self.audit_log(signal);

        Ok(())
    }

    fn audit_log(&self, signal: &Signal) {
        info!("Audit log: Signal {:?}", signal);
    }

    pub fn update_pnl(&mut self, pnl: Decimal) {
        self.daily_pnl += pnl;
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = dec!(0);
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

        checker.update_pnl(Decimal::from_f64_retain(-6000.0).unwrap());
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
        let checker = ComplianceChecker::new(&config);

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

        checker.update_pnl(Decimal::from_f64_retain(100.0).unwrap());
        assert_eq!(checker.daily_pnl, Decimal::from_f64_retain(100.0).unwrap());

        checker.update_pnl(Decimal::from_f64_retain(-50.0).unwrap());
        assert_eq!(checker.daily_pnl, Decimal::from_f64_retain(50.0).unwrap());
    }

    #[test]
    fn test_reset_daily() {
        let mut checker = create_test_compliance();

        checker.update_pnl(Decimal::from_f64_retain(100.0).unwrap());
        checker.reset_daily();

        assert_eq!(checker.daily_pnl, dec!(0));
    }
}
