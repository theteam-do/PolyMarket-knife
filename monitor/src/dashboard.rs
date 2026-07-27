//! 监控仪表板

use crate::alerts::AlertManager;
use crate::metrics::Metrics;
use tracing::info;

pub struct Dashboard {
    metrics: Metrics,
    alerts: AlertManager,
}

impl Dashboard {
    pub fn new(metrics: Metrics, alerts: AlertManager) -> Self {
        Self { metrics, alerts }
    }

    pub fn get_status(&self) -> DashboardStatus {
        DashboardStatus {
            daily_pnl: self.metrics.daily_pnl.get(),
            total_pnl: self.metrics.total_pnl.get(),
            orders_placed: self.metrics.orders_placed.get() as u64,
            orders_filled: self.metrics.orders_filled.get() as u64,
            opportunities: self.metrics.opportunities_found.get() as u64,
            should_stop: self.alerts.should_stop_trading(),
            alerts_count: self.alerts.get_alerts().len(),
        }
    }

    pub fn print_status(&self) {
        let status = self.get_status();

        info!("\n=== Trading Dashboard ===");
        info!("Daily PnL:      ${:.2}", status.daily_pnl);
        info!("Total PnL:      ${:.2}", status.total_pnl);
        info!(
            "Orders:         {} placed, {} filled",
            status.orders_placed, status.orders_filled
        );
        info!("Opportunities:  {}", status.opportunities);
        info!("Alerts:         {}", status.alerts_count);
        info!(
            "Status:         {}",
            if status.should_stop {
                "STOPPED"
            } else {
                "RUNNING"
            }
        );
        info!("========================\n");
    }
}

#[derive(Debug)]
pub struct DashboardStatus {
    pub daily_pnl: f64,
    pub total_pnl: f64,
    pub orders_placed: u64,
    pub orders_filled: u64,
    pub opportunities: u64,
    pub should_stop: bool,
    pub alerts_count: usize,
}
