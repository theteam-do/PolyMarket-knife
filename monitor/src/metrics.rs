//! 监控指标

use prometheus::{Counter, Gauge, Histogram, Registry, TextEncoder, Encoder};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use std::time::Instant;
use std::io::Write;

pub struct Metrics {
    registry: Registry,
    pub daily_pnl: Gauge,
    pub total_pnl: Counter,
    pub max_drawdown: Gauge,
    pub orders_placed: Counter,
    pub orders_filled: Counter,
    pub orders_cancelled: Counter,
    pub orders_failed: Counter,
    pub opportunities_found: Counter,
    pub opportunities_executed: Counter,
    pub signals_generated: Counter,
    pub total_position: Gauge,
    pub position_per_market: Gauge,
    pub api_latency: Histogram,
    pub order_latency: Histogram,
    pub consecutive_losses: Gauge,
    pub risk_exposure: Gauge,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        let daily_pnl = Gauge::new("daily_pnl_usd", "Daily PnL in USD").unwrap();
        let total_pnl = Counter::new("total_pnl_usd", "Total PnL in USD").unwrap();
        let max_drawdown = Gauge::new("max_drawdown_usd", "Maximum drawdown in USD").unwrap();
        let orders_placed = Counter::new("orders_placed_total", "Total orders placed").unwrap();
        let orders_filled = Counter::new("orders_filled_total", "Total orders filled").unwrap();
        let orders_cancelled = Counter::new("orders_cancelled_total", "Total orders cancelled").unwrap();
        let orders_failed = Counter::new("orders_failed_total", "Total orders failed").unwrap();
        let opportunities_found = Counter::new("opportunities_found_total", "Total opportunities found").unwrap();
        let opportunities_executed = Counter::new("opportunities_executed_total", "Total opportunities executed").unwrap();
        let signals_generated = Counter::new("signals_generated_total", "Total signals generated").unwrap();
        let total_position = Gauge::new("total_position_usd", "Total position in USD").unwrap();
        let position_per_market = Gauge::new("position_per_market_usd", "Position per market in USD").unwrap();
        let api_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new("api_latency_ms", "API latency in ms")
                .buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0])
        ).unwrap();
        let order_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new("order_latency_ms", "Order placement latency in ms")
                .buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0])
        ).unwrap();
        let consecutive_losses = Gauge::new("consecutive_losses", "Consecutive losses count").unwrap();
        let risk_exposure = Gauge::new("risk_exposure_usd", "Risk exposure in USD").unwrap();
        
        registry.register(Box::new(daily_pnl.clone())).unwrap();
        registry.register(Box::new(total_pnl.clone())).unwrap();
        registry.register(Box::new(max_drawdown.clone())).unwrap();
        registry.register(Box::new(orders_placed.clone())).unwrap();
        registry.register(Box::new(orders_filled.clone())).unwrap();
        registry.register(Box::new(orders_cancelled.clone())).unwrap();
        registry.register(Box::new(orders_failed.clone())).unwrap();
        registry.register(Box::new(opportunities_found.clone())).unwrap();
        registry.register(Box::new(opportunities_executed.clone())).unwrap();
        registry.register(Box::new(signals_generated.clone())).unwrap();
        registry.register(Box::new(total_position.clone())).unwrap();
        registry.register(Box::new(position_per_market.clone())).unwrap();
        registry.register(Box::new(api_latency.clone())).unwrap();
        registry.register(Box::new(order_latency.clone())).unwrap();
        registry.register(Box::new(consecutive_losses.clone())).unwrap();
        registry.register(Box::new(risk_exposure.clone())).unwrap();
        
        Self {
            registry, daily_pnl, total_pnl, max_drawdown, orders_placed, orders_filled,
            orders_cancelled, orders_failed, opportunities_found, opportunities_executed,
            signals_generated, total_position, position_per_market, api_latency, order_latency,
            consecutive_losses, risk_exposure,
        }
    }
    
    pub fn record_order(&self, status: OrderStatus, latency_ms: f64) {
        self.orders_placed.inc();
        self.order_latency.observe(latency_ms);
        match status {
            OrderStatus::Filled => self.orders_filled.inc(),
            OrderStatus::Cancelled => self.orders_cancelled.inc(),
            OrderStatus::Failed => self.orders_failed.inc(),
        }
    }
    
    pub fn record_pnl(&self, pnl: Decimal) {
        let pnl_f64 = pnl.to_string().parse::<f64>().unwrap_or(0.0);
        self.daily_pnl.set(pnl_f64);
        self.total_pnl.inc_by(pnl_f64.abs());
        if pnl < dec!(0) {
            self.max_drawdown.set(pnl_f64.abs());
        }
    }
    
    pub fn record_api_latency(&self, latency_ms: f64) {
        self.api_latency.observe(latency_ms);
    }
    
    pub fn record_position(&self, total: f64, per_market: f64) {
        self.total_position.set(total);
        self.position_per_market.set(per_market);
    }
    
    pub fn record_risk(&self, consecutive: u32, exposure: f64) {
        self.consecutive_losses.set(consecutive as f64);
        self.risk_exposure.set(exposure);
    }
    
    pub fn gather(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Filled,
    Cancelled,
    Failed,
}

pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        
        assert_eq!(metrics.daily_pnl.get(), 0.0);
        assert_eq!(metrics.total_pnl.get(), 0.0);
        assert_eq!(metrics.orders_placed.get(), 0.0);
    }

    #[test]
    fn test_record_order() {
        let metrics = Metrics::new();
        
        metrics.record_order(OrderStatus::Filled, 50.0);
        
        assert_eq!(metrics.orders_placed.get(), 1.0);
        assert_eq!(metrics.orders_filled.get(), 1.0);
    }

    #[test]
    fn test_record_pnl() {
        let metrics = Metrics::new();
        
        metrics.record_pnl(dec!(100.50));
        
        assert!(metrics.daily_pnl.get() > 100.0);
        assert!(metrics.daily_pnl.get() < 101.0);
    }

    #[test]
    fn test_record_loss() {
        let metrics = Metrics::new();
        
        metrics.record_pnl(dec!(-50.0));
        
        assert!(metrics.daily_pnl.get() < -49.0);
        assert!(metrics.max_drawdown.get() > 49.0);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new();
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10.0);
        assert!(elapsed < 100.0);
    }

    #[test]
    fn test_gather_metrics() {
        let metrics = Metrics::new();
        
        metrics.orders_placed.inc();
        metrics.orders_filled.inc();
        
        let output = metrics.gather();
        
        assert!(output.contains("orders_placed_total"));
        assert!(output.contains("orders_filled_total"));
    }
}
