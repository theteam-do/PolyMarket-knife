use anyhow::Result;
use monitor::{AlertManager, Dashboard, Metrics};

#[tokio::main]
async fn main() -> Result<()> {
    let metrics = Metrics::new();
    let alerts = AlertManager::default();

    println!("=== Monitoring Demo ===");

    metrics.orders_placed.inc();
    metrics.orders_filled.inc();

    let status = Dashboard::new(metrics, alerts).get_status();
    println!("Orders placed: {}", status.orders_placed);
    println!("Orders filled: {}", status.orders_filled);

    Ok(())
}
