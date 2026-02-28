//! 风控管理器

use crate::config::StrategyConfig;
use crate::monitor::TradeEvent;
use std::collections::HashMap;

pub struct RiskManager {
    config: StrategyConfig,
    daily_pnl: f64,
    positions: HashMap<String, f64>, // market_id -> position_usd
}

impl RiskManager {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
            daily_pnl: 0.0,
            positions: HashMap::new(),
        }
    }

    pub fn can_trade(&self, trade: &TradeEvent) -> bool {
        // 检查黑名单
        if self.config.blacklist.contains(&trade.market) {
            return false;
        }

        // 检查日亏损
        if self.daily_pnl < -self.config.max_daily_loss {
            return false;
        }

        // 检查最小交易规模
        let copy_size = trade.size_usd * self.config.copy_ratio;
        if copy_size < self.config.min_trade_size_usd {
            return false;
        }

        // 检查市场持仓限制
        if let Some(pos) = self.positions.get(&trade.market_id) {
            if pos + copy_size > self.config.max_position_per_market {
                return false;
            }
        }

        true
    }

    pub fn update_position(&mut self, market_id: &str, delta: f64) {
        let pos = self.positions.entry(market_id.to_string()).or_insert(0.0);
        *pos += delta;
    }

    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
    }
}
