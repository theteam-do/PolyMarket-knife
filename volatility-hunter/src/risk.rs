//! 风控管理器

use crate::config::StrategyConfig;
use crate::signal::Signal;
use std::collections::HashMap;

pub struct RiskManager {
    config: StrategyConfig,
    daily_pnl: f64,
    positions: HashMap<String, f64>,
    consecutive_losses: u32,
}

impl RiskManager {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
            daily_pnl: 0.0,
            positions: HashMap::new(),
            consecutive_losses: 0,
        }
    }

    pub fn can_trade(&self, signal: &Signal) -> bool {
        // 检查日亏损
        if self.daily_pnl < -self.config.max_daily_loss {
            return false;
        }

        // 检查连续亏损 (防止上头)
        if self.consecutive_losses >= 5 {
            return false;
        }

        // 高置信度信号总是可以交易
        if signal.confidence() >= self.config.confidence_high {
            return true;
        }

        // 低置信度信号需要更严格检查
        signal.confidence() >= 0.5
    }

    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;

        if pnl < 0.0 {
            self.consecutive_losses += 1;
        } else {
            self.consecutive_losses = 0;
        }
    }

    pub fn update_position(&mut self, market: &str, delta: f64) {
        let pos = self.positions.entry(market.to_string()).or_insert(0.0);
        *pos += delta;
    }

    pub fn daily_pnl(&self) -> f64 {
        self.daily_pnl
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
        self.consecutive_losses = 0;
    }
}
