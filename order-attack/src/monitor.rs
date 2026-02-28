//! 订单簿监控器

use crate::config::Config;

pub struct OrderbookMonitor {
    #[allow(dead_code)]
    config: Config,
}

impl OrderbookMonitor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn wait_for_clearing(&self, _market: &str) -> bool {
        false
    }
}
