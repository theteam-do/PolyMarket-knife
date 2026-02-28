//! 风控管理器

use crate::config::RiskConfig;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Position {
    pub yes_size: f64,
    pub no_size: f64,
    pub avg_cost: f64,
}

pub struct RiskManager {
    config: RiskConfig,
    can_trade: bool,
    positions: HashMap<String, Position>,
    daily_pnl: f64,
    daily_volume: f64,
}

impl RiskManager {
    pub fn new(config: &RiskConfig) -> Self {
        Self {
            config: config.clone(),
            can_trade: true,
            positions: HashMap::new(),
            daily_pnl: 0.0,
            daily_volume: 0.0,
        }
    }

    pub fn can_trade(&self) -> bool {
        self.can_trade && self.daily_pnl > -self.config.max_loss_per_day
    }

    pub fn can_place_order(&self, market_id: &str, _bid: f64, _ask: f64) -> bool {
        // 检查总持仓
        let total_position = self.total_position_value();
        let order_value = 1000.0; // 默认订单大小

        if total_position + order_value > self.config.max_position_usd {
            return false;
        }

        // 检查单个市场持仓
        if let Some(pos) = self.positions.get(market_id) {
            let market_value = pos.yes_size + pos.no_size;
            if market_value + order_value > self.config.max_position_usd * 0.3 {
                return false;
            }
        }

        true
    }

    pub fn total_position_value(&self) -> f64 {
        self.positions
            .values()
            .map(|p| p.yes_size + p.no_size)
            .sum()
    }

    pub fn get_position(&self, market_id: &str) -> Option<&Position> {
        self.positions.get(market_id)
    }

    pub fn update_position(&mut self, market_id: &str, yes_delta: f64, no_delta: f64, price: f64) {
        let pos = self.positions.entry(market_id.to_string()).or_default();
        pos.yes_size += yes_delta;
        pos.no_size += no_delta;

        // 更新平均成本
        let trade_value = (yes_delta + no_delta) * price;
        if trade_value > 0.0 {
            pos.avg_cost = (pos.avg_cost * (pos.yes_size + pos.no_size - trade_value)
                + trade_value * price)
                / (pos.yes_size + pos.no_size);
        }
    }

    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
    }

    pub fn update_volume(&mut self, volume: f64) {
        self.daily_volume += volume;
    }

    pub fn daily_pnl(&self) -> f64 {
        self.daily_pnl
    }

    pub fn daily_volume(&self) -> f64 {
        self.daily_volume
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
        self.daily_volume = 0.0;
    }

    pub fn stop_trading(&mut self) {
        self.can_trade = false;
    }

    pub fn resume_trading(&mut self) {
        self.can_trade = true;
    }
}
