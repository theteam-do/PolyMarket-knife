//! 监控指标 - Prometheus 格式

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 监控指标收集器
pub struct MetricsCollector {
    orders_placed: AtomicU64,
    orders_filled: AtomicU64,
    orders_cancelled: AtomicU64,
    orders_failed: AtomicU64,
    daily_pnl: AtomicI64,
    daily_volume: AtomicU64,
    last_update: AtomicU64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            orders_placed: AtomicU64::new(0),
            orders_filled: AtomicU64::new(0),
            orders_cancelled: AtomicU64::new(0),
            orders_failed: AtomicU64::new(0),
            daily_pnl: AtomicI64::new(0),
            daily_volume: AtomicU64::new(0),
            last_update: AtomicU64::new(current_timestamp()),
        }
    }

    pub fn record_placed(&self, count: u64) {
        self.orders_placed.fetch_add(count, Ordering::Relaxed);
        self.touch();
    }

    pub fn record_filled(&self, count: u64) {
        self.orders_filled.fetch_add(count, Ordering::Relaxed);
        self.touch();
    }

    pub fn record_cancelled(&self, count: u64) {
        self.orders_cancelled.fetch_add(count, Ordering::Relaxed);
        self.touch();
    }

    pub fn record_failed(&self, count: u64) {
        self.orders_failed.fetch_add(count, Ordering::Relaxed);
        self.touch();
    }

    pub fn record_pnl(&self, pnl: Decimal) {
        let pnl_cents = (pnl * Decimal::from(100))
            .round()
            .to_i64()
            .unwrap_or_else(|| {
                tracing::warn!("PnL value out of range: {}", pnl);
                0
            });
        self.daily_pnl.fetch_add(pnl_cents, Ordering::Relaxed);
        self.touch();
    }

    pub fn record_volume(&self, volume: Decimal) {
        let volume_cents = (volume * Decimal::from(100))
            .round()
            .to_u64()
            .unwrap_or_else(|| {
                tracing::warn!("Volume value out of range: {}", volume);
                0
            });
        self.daily_volume.fetch_add(volume_cents, Ordering::Relaxed);
        self.touch();
    }

    pub fn export_prometheus(&self) -> String {
        let timestamp = self.last_update.load(Ordering::Relaxed);

        format!(
            r#"# HELP market_maker_orders_placed Total orders placed
# TYPE market_maker_orders_placed counter
market_maker_orders_placed {} {}

# HELP market_maker_orders_filled Total orders filled
# TYPE market_maker_orders_filled counter
market_maker_orders_filled {} {}

# HELP market_maker_orders_cancelled Total orders cancelled
# TYPE market_maker_orders_cancelled counter
market_maker_orders_cancelled {} {}

# HELP market_maker_orders_failed Total orders failed
# TYPE market_maker_orders_failed counter
market_maker_orders_failed {} {}

# HELP market_maker_daily_pnl Daily PnL in cents
# TYPE market_maker_daily_pnl gauge
market_maker_daily_pnl {} {}

# HELP market_maker_daily_volume Daily volume in cents
# TYPE market_maker_daily_volume counter
market_maker_daily_volume {} {}

# HELP market_maker_last_update Last update timestamp
# TYPE market_maker_last_update gauge
market_maker_last_update {} {}
"#,
            self.orders_placed.load(Ordering::Relaxed),
            timestamp,
            self.orders_filled.load(Ordering::Relaxed),
            timestamp,
            self.orders_cancelled.load(Ordering::Relaxed),
            timestamp,
            self.orders_failed.load(Ordering::Relaxed),
            timestamp,
            self.daily_pnl.load(Ordering::Relaxed),
            timestamp,
            self.daily_volume.load(Ordering::Relaxed),
            timestamp,
            timestamp,
            timestamp,
        )
    }

    pub fn reset_daily(&self) {
        self.daily_pnl.store(0, Ordering::Relaxed);
        self.daily_volume.store(0, Ordering::Relaxed);
        self.touch();
    }

    fn touch(&self) {
        self.last_update
            .store(current_timestamp(), Ordering::Relaxed);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before UNIX epoch")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_record_order_counters() {
        let collector = MetricsCollector::new();

        collector.record_placed(2);
        collector.record_cancelled(1);
        collector.record_failed(1);
        collector.record_filled(1);

        assert_eq!(collector.orders_placed.load(Ordering::Relaxed), 2);
        assert_eq!(collector.orders_cancelled.load(Ordering::Relaxed), 1);
        assert_eq!(collector.orders_failed.load(Ordering::Relaxed), 1);
        assert_eq!(collector.orders_filled.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_pnl() {
        let collector = MetricsCollector::new();

        collector.record_pnl(dec!(100.50));
        collector.record_pnl(dec!(-50.25));

        assert_eq!(collector.daily_pnl.load(Ordering::Relaxed), 5025);
    }

    #[test]
    fn test_export_prometheus() {
        let collector = MetricsCollector::new();
        collector.record_placed(2);
        collector.record_volume(dec!(12.34));

        let output = collector.export_prometheus();

        assert!(output.contains("market_maker_orders_placed 2"));
        assert!(output.contains("market_maker_daily_volume 1234"));
    }

    #[test]
    fn test_reset_daily() {
        let collector = MetricsCollector::new();

        collector.record_pnl(dec!(100.0));
        collector.reset_daily();

        assert_eq!(collector.daily_pnl.load(Ordering::Relaxed), 0);
    }
}
