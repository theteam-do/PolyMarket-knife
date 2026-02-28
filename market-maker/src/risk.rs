//! 风控管理器 - 生产级实现

use std::collections::HashMap;
use tracing::info;

/// 持仓信息
#[derive(Debug, Default, Clone)]
pub struct Position {
    pub yes_size: f64,
    pub no_size: f64,
    pub avg_cost: f64,
}

/// 风控管理器
pub struct RiskManager {
    config: RiskConfig,
    can_trade_flag: bool,
    positions: HashMap<String, Position>,
    daily_pnl: f64,
    daily_volume: f64,
}

/// 风控配置
#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_position_usd: f64,
    pub max_loss_per_day: f64,
    pub stop_loss_pct: f64,
    pub max_orders: usize,
    pub max_order_size_usd: f64,
}

impl RiskManager {
    /// 创建新的风控管理器
    pub fn new(config: &crate::config::RiskConfig) -> Self {
        Self {
            config: RiskConfig {
                max_position_usd: config.max_position_usd,
                max_loss_per_day: config.max_loss_per_day,
                stop_loss_pct: config.stop_loss_pct,
                max_orders: config.max_orders,
                max_order_size_usd: config.max_order_size_usd,
            },
            can_trade_flag: true,
            positions: HashMap::new(),
            daily_pnl: 0.0,
            daily_volume: 0.0,
        }
    }

    /// 检查是否可以交易
    pub fn can_trade(&self) -> bool {
        let _ = self.config.stop_loss_pct;
        let _ = self.config.max_orders;

        if !self.can_trade_flag {
            return false;
        }

        // 检查日亏损
        if self.daily_pnl < -self.config.max_loss_per_day {
            return false;
        }

        true
    }

    /// 检查是否可以下单
    pub fn can_place_order(&self, market_id: &str, size: f64) -> bool {
        // 检查订单大小
        if size > self.config.max_order_size_usd {
            return false;
        }

        // 检查总持仓
        let total_position = self.total_position_value();
        if total_position + size > self.config.max_position_usd {
            return false;
        }

        // 检查单市场持仓
        if let Some(pos) = self.positions.get(market_id) {
            let market_value = pos.yes_size + pos.no_size;
            if market_value + size > self.config.max_position_usd * 0.3 {
                return false;
            }
        }

        true
    }

    /// 更新持仓
    pub fn update_position(&mut self, market_id: &str, yes_delta: f64, no_delta: f64, price: f64) {
        let pos = self.positions.entry(market_id.to_string()).or_default();
        pos.yes_size += yes_delta;
        pos.no_size += no_delta;

        // 更新平均成本
        let trade_value = (yes_delta + no_delta) * price;
        if trade_value > 0.0 {
            let total_value = pos.yes_size + pos.no_size;
            if total_value > 0.0 {
                pos.avg_cost = (pos.avg_cost * (total_value - trade_value) + trade_value * price)
                    / total_value;
            }
        }

        info!(
            "Position updated for {}: yes={}, no={}, avg_cost={}",
            market_id, pos.yes_size, pos.no_size, pos.avg_cost
        );
    }

    /// 获取持仓
    pub fn get_position(&self, market_id: &str) -> Option<&Position> {
        self.positions.get(market_id)
    }

    /// 更新 PnL
    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
        info!("Daily PnL updated: {}", self.daily_pnl);
    }

    /// 更新成交量
    pub fn update_volume(&mut self, volume: f64) {
        self.daily_volume += volume;
    }

    /// 获取日 PnL
    pub fn daily_pnl(&self) -> f64 {
        self.daily_pnl
    }

    /// 获取日成交量
    pub fn daily_volume(&self) -> f64 {
        self.daily_volume
    }

    /// 重置日统计
    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
        self.daily_volume = 0.0;
        info!("Daily statistics reset");
    }

    /// 停止交易
    pub fn stop_trading(&mut self) {
        self.can_trade_flag = false;
        info!("Trading stopped");
    }

    /// 恢复交易
    pub fn resume_trading(&mut self) {
        self.can_trade_flag = true;
        info!("Trading resumed");
    }

    /// 获取总持仓价值
    pub fn total_position_value(&self) -> f64 {
        self.positions
            .values()
            .map(|p| p.yes_size + p.no_size)
            .sum()
    }
}
