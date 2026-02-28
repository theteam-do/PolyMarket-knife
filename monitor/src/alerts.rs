//! 告警管理

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::time::{Duration, Instant};
use tracing::{error, warn};

/// 告警管理器
pub struct AlertManager {
    config: AlertConfig,
    alerts: Vec<Alert>,
    last_alert_time: Option<Instant>,
}

/// 告警配置
#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub daily_loss_threshold: Decimal,
    pub daily_loss_warning_pct: f64,
    pub position_threshold: Decimal,
    pub api_error_rate_threshold: f64,
    pub latency_threshold_ms: f64,
    pub consecutive_loss_threshold: u32,
    pub cooldown_duration: Duration,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            daily_loss_threshold: dec!(500),
            daily_loss_warning_pct: 0.8,
            position_threshold: dec!(10000),
            api_error_rate_threshold: 0.1,
            latency_threshold_ms: 100.0,
            consecutive_loss_threshold: 5,
            cooldown_duration: Duration::from_secs(60),
        }
    }
}

/// 告警类型
#[derive(Debug, Clone)]
pub enum AlertType {
    DailyLossWarning {
        current: Decimal,
        threshold: Decimal,
    },
    DailyLossExceeded {
        current: Decimal,
        threshold: Decimal,
    },
    PositionExceeded {
        current: Decimal,
        threshold: Decimal,
    },
    HighErrorRate {
        rate: f64,
        threshold: f64,
    },
    HighLatency {
        latency_ms: f64,
        threshold_ms: f64,
    },
    ConsecutiveLosses {
        count: u32,
        threshold: u32,
    },
    OrderFailed {
        order_id: String,
        reason: String,
    },
}

/// 告警
#[derive(Debug, Clone)]
pub struct Alert {
    pub alert_type: AlertType,
    pub severity: Severity,
    pub message: String,
    pub timestamp: Instant,
}

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl AlertManager {
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            alerts: Vec::new(),
            last_alert_time: None,
        }
    }

    pub fn check_daily_loss(&mut self, current_pnl: Decimal) -> Vec<Alert> {
        let mut alerts = Vec::new();

        let threshold = self.config.daily_loss_threshold;
        let warning_threshold =
            threshold * Decimal::from_f64_retain(self.config.daily_loss_warning_pct).unwrap();

        if current_pnl < -threshold {
            let msg = format!("日亏损超限！当前：${}, 阈值：${}", current_pnl, threshold);
            error!("CRITICAL: {}", msg);
            alerts.push(Alert {
                alert_type: AlertType::DailyLossExceeded {
                    current: current_pnl,
                    threshold,
                },
                severity: Severity::Critical,
                message: msg,
                timestamp: Instant::now(),
            });
        } else if current_pnl < -warning_threshold {
            let msg = format!(
                "日亏损接近阈值！当前：${}, 警告线：${}",
                current_pnl, warning_threshold
            );
            warn!("WARNING: {}", msg);
            alerts.push(Alert {
                alert_type: AlertType::DailyLossWarning {
                    current: current_pnl,
                    threshold: warning_threshold,
                },
                severity: Severity::Warning,
                message: msg,
                timestamp: Instant::now(),
            });
        }

        self.record_alerts(alerts)
    }

    pub fn check_position(&mut self, current_position: Decimal) -> Vec<Alert> {
        let mut alerts = Vec::new();

        if current_position > self.config.position_threshold {
            let msg = format!(
                "持仓超限！当前：${}, 阈值：${}",
                current_position, self.config.position_threshold
            );
            error!("CRITICAL: {}", msg);
            alerts.push(Alert {
                alert_type: AlertType::PositionExceeded {
                    current: current_position,
                    threshold: self.config.position_threshold,
                },
                severity: Severity::Critical,
                message: msg,
                timestamp: Instant::now(),
            });
        }

        self.record_alerts(alerts)
    }

    pub fn check_latency(&mut self, latency_ms: f64) -> Vec<Alert> {
        let mut alerts = Vec::new();

        if latency_ms > self.config.latency_threshold_ms {
            let msg = format!(
                "延迟过高！当前：{}ms, 阈值：{}ms",
                latency_ms, self.config.latency_threshold_ms
            );
            warn!("WARNING: {}", msg);
            alerts.push(Alert {
                alert_type: AlertType::HighLatency {
                    latency_ms,
                    threshold_ms: self.config.latency_threshold_ms,
                },
                severity: Severity::Warning,
                message: msg,
                timestamp: Instant::now(),
            });
        }

        self.record_alerts(alerts)
    }

    pub fn check_consecutive_losses(&mut self, count: u32) -> Vec<Alert> {
        let mut alerts = Vec::new();

        if count >= self.config.consecutive_loss_threshold {
            let msg = format!(
                "连续亏损过多！当前：{}次，阈值：{}次",
                count, self.config.consecutive_loss_threshold
            );
            error!("CRITICAL: {}", msg);
            alerts.push(Alert {
                alert_type: AlertType::ConsecutiveLosses {
                    count,
                    threshold: self.config.consecutive_loss_threshold,
                },
                severity: Severity::Critical,
                message: msg,
                timestamp: Instant::now(),
            });
        }

        self.record_alerts(alerts)
    }

    pub fn record_order_failure(&mut self, order_id: &str, reason: &str) -> Vec<Alert> {
        let msg = format!("订单失败！ID: {}, 原因：{}", order_id, reason);
        warn!("WARNING: {}", msg);
        let alert = Alert {
            alert_type: AlertType::OrderFailed {
                order_id: order_id.to_string(),
                reason: reason.to_string(),
            },
            severity: Severity::Warning,
            message: msg,
            timestamp: Instant::now(),
        };
        self.record_alerts(vec![alert])
    }

    fn record_alerts(&mut self, alerts: Vec<Alert>) -> Vec<Alert> {
        if let Some(last_time) = self.last_alert_time {
            if last_time.elapsed() < self.config.cooldown_duration {
                return Vec::new();
            }
        }

        if !alerts.is_empty() {
            self.last_alert_time = Some(Instant::now());
            for alert in &alerts {
                self.alerts.push(alert.clone());
            }
        }

        alerts
    }

    pub fn get_alerts(&self) -> &[Alert] {
        &self.alerts
    }

    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    pub fn should_stop_trading(&self) -> bool {
        self.alerts.iter().any(|a| a.severity == Severity::Critical)
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new(AlertConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_alert_config_default() {
        let config = AlertConfig::default();

        assert_eq!(config.daily_loss_threshold, dec!(500));
        assert_eq!(config.daily_loss_warning_pct, 0.8);
        assert_eq!(config.position_threshold, dec!(10000));
        assert_eq!(config.latency_threshold_ms, 100.0);
        assert_eq!(config.consecutive_loss_threshold, 5);
    }

    #[test]
    fn test_daily_loss_warning() {
        let mut alerts = AlertManager::default();

        let pnl = dec!(-450.0); // 90% of 500 threshold
        let alert_list = alerts.check_daily_loss(pnl);

        assert!(!alert_list.is_empty());
        assert_eq!(alert_list[0].severity, Severity::Warning);
    }

    #[test]
    fn test_daily_loss_exceeded() {
        let mut alerts = AlertManager::default();

        let pnl = dec!(-550.0); // Exceeds 500 threshold
        let alert_list = alerts.check_daily_loss(pnl);

        assert!(!alert_list.is_empty());
        assert_eq!(alert_list[0].severity, Severity::Critical);
    }

    #[test]
    fn test_should_stop_trading() {
        let mut alerts = AlertManager::default();

        // No critical alerts yet
        assert!(!alerts.should_stop_trading());

        // Trigger critical alert
        alerts.check_daily_loss(dec!(-550.0));

        assert!(alerts.should_stop_trading());
    }

    #[test]
    fn test_alert_cooldown() {
        let mut alerts = AlertManager::default();

        // First alert should trigger
        let pnl = dec!(-550.0);
        let first_alerts = alerts.check_daily_loss(pnl);
        assert!(!first_alerts.is_empty());

        // Second alert within cooldown should not trigger
        let second_alerts = alerts.check_daily_loss(pnl);
        assert!(second_alerts.is_empty());
    }

    #[test]
    fn test_consecutive_losses() {
        let mut alerts = AlertManager::default();

        // 4 losses should not trigger
        let alerts_list = alerts.check_consecutive_losses(4);
        assert!(alerts_list.is_empty());

        // 5 losses should trigger
        let alerts_list = alerts.check_consecutive_losses(5);
        assert!(!alerts_list.is_empty());
        assert_eq!(alerts_list[0].severity, Severity::Critical);
    }

    #[test]
    fn test_clear_alerts() {
        let mut alerts = AlertManager::default();

        alerts.check_daily_loss(dec!(-550.0));
        assert!(!alerts.get_alerts().is_empty());

        alerts.clear_alerts();
        assert!(alerts.get_alerts().is_empty());
    }
}
