//! 风控管理器 - 生产级实现

use std::collections::HashMap;
use tracing::info;

const EPSILON: f64 = 1e-9;

#[derive(Debug, Default, Clone)]
pub struct Position {
    pub net_shares: f64,
    pub avg_cost: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenOrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct OpenOrderReservation {
    market_id: String,
    side: OpenOrderSide,
    order_price: f64,
    original_shares: f64,
    matched_shares: f64,
}

impl OpenOrderReservation {
    fn remaining_shares(&self) -> f64 {
        (self.original_shares - self.matched_shares).max(0.0)
    }

    fn remaining_notional_usd(&self) -> f64 {
        self.remaining_shares() * self.order_price
    }

    fn is_fully_filled(&self) -> bool {
        self.remaining_shares() <= EPSILON
    }
}

#[derive(Debug, Clone)]
pub struct FillEffect {
    pub order_id: String,
    pub market_id: String,
    pub fill_shares: f64,
    pub fill_price: f64,
    pub fill_notional_usd: f64,
    pub realized_pnl_delta: f64,
    pub order_completed: bool,
    pub net_position_shares: f64,
    pub remaining_open_notional_usd: f64,
}

pub struct RiskManager {
    config: RiskConfig,
    can_trade_flag: bool,
    positions: HashMap<String, Position>,
    open_orders: HashMap<String, OpenOrderReservation>,
    daily_pnl: f64,
    daily_volume: f64,
}

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_position_usd: f64,
    pub max_loss_per_day: f64,
    #[allow(dead_code)]
    pub stop_loss_pct: f64,
    pub max_orders: usize,
    pub max_order_size_usd: f64,
    pub max_market_concentration_pct: f64,
}

impl RiskManager {
    pub fn new(config: &crate::config::RiskConfig) -> Self {
        Self {
            config: RiskConfig {
                max_position_usd: config.max_position_usd,
                max_loss_per_day: config.max_loss_per_day,
                stop_loss_pct: config.stop_loss_pct,
                max_orders: config.max_orders,
                max_order_size_usd: config.max_order_size_usd,
                max_market_concentration_pct: config.max_market_concentration_pct,
            },
            can_trade_flag: true,
            positions: HashMap::new(),
            open_orders: HashMap::new(),
            daily_pnl: 0.0,
            daily_volume: 0.0,
        }
    }

    pub fn can_trade(&self) -> bool {
        if !self.can_trade_flag {
            return false;
        }

        self.daily_pnl >= -self.config.max_loss_per_day
    }

    pub fn can_place_orders(&self, market_id: &str, notionals_usd: &[f64]) -> bool {
        if notionals_usd.is_empty() {
            return false;
        }

        let mut proposed_total = 0.0;
        for notional in notionals_usd {
            if *notional <= 0.0 || *notional > self.config.max_order_size_usd {
                return false;
            }
            proposed_total += *notional;
        }

        if self.open_orders.len() + notionals_usd.len() > self.config.max_orders {
            return false;
        }

        let total_exposure = self.total_position_value() + self.total_open_order_value();
        if total_exposure + proposed_total > self.config.max_position_usd {
            return false;
        }

        let market_exposure =
            self.market_position_value(market_id) + self.market_open_order_value(market_id);
        let concentration_limit = self.config.max_position_usd * self.config.max_market_concentration_pct;
        if market_exposure + proposed_total > concentration_limit {
            return false;
        }

        true
    }

    pub fn reserve_open_order(
        &mut self,
        order_id: &str,
        market_id: &str,
        side: OpenOrderSide,
        order_price: f64,
        original_shares: f64,
    ) {
        if order_price <= 0.0 || original_shares <= 0.0 {
            return;
        }

        self.open_orders.insert(
            order_id.to_string(),
            OpenOrderReservation {
                market_id: market_id.to_string(),
                side,
                order_price,
                original_shares,
                matched_shares: 0.0,
            },
        );
        info!(
            "Reserved open-order exposure: order_id={} market={} side={:?} shares={:.4} price={:.6} notional_usd={:.4}",
            order_id,
            market_id,
            side,
            original_shares,
            order_price,
            original_shares * order_price
        );
    }

    pub fn release_open_order(&mut self, order_id: &str) -> Option<f64> {
        let reservation = self.open_orders.remove(order_id)?;
        let released_notional = reservation.remaining_notional_usd();
        info!(
            "Released open-order exposure: order_id={} market={} side={:?} released_notional_usd={:.4}",
            order_id,
            reservation.market_id,
            reservation.side,
            released_notional
        );
        Some(released_notional)
    }

    pub fn apply_fill(
        &mut self,
        order_id: &str,
        fill_shares: f64,
        fill_price: f64,
    ) -> Option<FillEffect> {
        if fill_shares <= EPSILON || fill_price <= EPSILON {
            return None;
        }

        let (market_id, side, applied_fill_shares, remaining_open_notional_usd, order_completed) = {
            let reservation = self.open_orders.get_mut(order_id)?;
            let remaining_shares = reservation.remaining_shares();
            let applied_fill_shares = fill_shares.min(remaining_shares);
            if applied_fill_shares <= EPSILON {
                return None;
            }

            reservation.matched_shares += applied_fill_shares;
            let order_completed = reservation.is_fully_filled();
            let remaining_open_notional_usd = reservation.remaining_notional_usd();

            (
                reservation.market_id.clone(),
                reservation.side,
                applied_fill_shares,
                remaining_open_notional_usd,
                order_completed,
            )
        };

        if order_completed {
            self.open_orders.remove(order_id);
        }

        let realized_pnl_delta =
            self.apply_trade_to_position(&market_id, side, applied_fill_shares, fill_price);
        let fill_notional_usd = applied_fill_shares * fill_price;
        self.daily_volume += fill_notional_usd;
        self.daily_pnl += realized_pnl_delta;

        let net_position_shares = self
            .positions
            .get(&market_id)
            .map(|position| position.net_shares)
            .unwrap_or_default();

        Some(FillEffect {
            order_id: order_id.to_string(),
            market_id,
            fill_shares: applied_fill_shares,
            fill_price,
            fill_notional_usd,
            realized_pnl_delta,
            order_completed,
            net_position_shares,
            remaining_open_notional_usd,
        })
    }

    pub fn clear_open_orders(&mut self) {
        self.open_orders.clear();
        info!("Open-order exposure cleared");
    }

    pub fn inventory_skew_signal(&self, market_id: &str) -> f64 {
        let Some(position) = self.positions.get(market_id) else {
            return 0.0;
        };
        if position.net_shares.abs() <= EPSILON || position.avg_cost <= EPSILON {
            return 0.0;
        }

        let notional = position.net_shares.abs() * position.avg_cost;
        let scale = (self.config.max_position_usd * self.config.max_market_concentration_pct).max(EPSILON);
        let magnitude = (notional / scale).clamp(0.0, 1.0);
        position.net_shares.signum() * magnitude
    }

    #[allow(dead_code)]
    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
        info!("Daily PnL updated: {}", self.daily_pnl);
    }

    pub fn daily_pnl(&self) -> f64 {
        self.daily_pnl
    }

    #[allow(dead_code)]
    pub fn daily_volume(&self) -> f64 {
        self.daily_volume
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
        self.daily_volume = 0.0;
        info!("Daily statistics reset");
    }

    pub fn stop_trading(&mut self) {
        self.can_trade_flag = false;
        info!("Trading stopped");
    }

    pub fn resume_trading(&mut self) {
        self.can_trade_flag = true;
        info!("Trading resumed");
    }

    pub fn total_position_value(&self) -> f64 {
        self.positions
            .values()
            .map(|position| position.net_shares.abs() * position.avg_cost)
            .sum()
    }

    pub fn total_open_order_value(&self) -> f64 {
        self.open_orders
            .values()
            .map(OpenOrderReservation::remaining_notional_usd)
            .sum()
    }

    pub fn market_position_value(&self, market_id: &str) -> f64 {
        self.positions
            .get(market_id)
            .map(|position| position.net_shares.abs() * position.avg_cost)
            .unwrap_or_default()
    }

    pub fn market_open_order_value(&self, market_id: &str) -> f64 {
        self.open_orders
            .values()
            .filter(|reservation| reservation.market_id == market_id)
            .map(OpenOrderReservation::remaining_notional_usd)
            .sum()
    }

    pub fn open_order_count(&self) -> usize {
        self.open_orders.len()
    }

    fn apply_trade_to_position(
        &mut self,
        market_id: &str,
        side: OpenOrderSide,
        fill_shares: f64,
        fill_price: f64,
    ) -> f64 {
        let position = self.positions.entry(market_id.to_string()).or_default();
        let mut remaining = fill_shares;
        let mut realized_pnl = 0.0;

        match side {
            OpenOrderSide::Buy => {
                if position.net_shares < -EPSILON {
                    let closing = remaining.min(-position.net_shares);
                    realized_pnl += (position.avg_cost - fill_price) * closing;
                    position.net_shares += closing;
                    remaining -= closing;
                    if position.net_shares.abs() <= EPSILON {
                        position.net_shares = 0.0;
                        position.avg_cost = 0.0;
                    }
                }

                if remaining > EPSILON {
                    if position.net_shares > EPSILON {
                        let existing = position.net_shares;
                        position.avg_cost = ((position.avg_cost * existing)
                            + (fill_price * remaining))
                            / (existing + remaining);
                        position.net_shares += remaining;
                    } else {
                        position.net_shares = remaining;
                        position.avg_cost = fill_price;
                    }
                }
            }
            OpenOrderSide::Sell => {
                if position.net_shares > EPSILON {
                    let closing = remaining.min(position.net_shares);
                    realized_pnl += (fill_price - position.avg_cost) * closing;
                    position.net_shares -= closing;
                    remaining -= closing;
                    if position.net_shares.abs() <= EPSILON {
                        position.net_shares = 0.0;
                        position.avg_cost = 0.0;
                    }
                }

                if remaining > EPSILON {
                    if position.net_shares < -EPSILON {
                        let existing = -position.net_shares;
                        position.avg_cost = ((position.avg_cost * existing)
                            + (fill_price * remaining))
                            / (existing + remaining);
                        position.net_shares -= remaining;
                    } else {
                        position.net_shares = -remaining;
                        position.avg_cost = fill_price;
                    }
                }
            }
        }

        position.realized_pnl += realized_pnl;
        info!(
            "Position updated from fill: market={} side={:?} fill_shares={:.4} fill_price={:.6} net_shares={:.4} avg_cost={:.6} realized_pnl_delta={:.4}",
            market_id,
            side,
            fill_shares,
            fill_price,
            position.net_shares,
            position.avg_cost,
            realized_pnl
        );

        realized_pnl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> crate::config::RiskConfig {
        crate::config::RiskConfig {
            max_position_usd: 1_000.0,
            max_loss_per_day: 100.0,
            stop_loss_pct: 5.0,
            max_orders: 3,
            max_order_size_usd: 250.0,
            max_market_concentration_pct: 0.3,
        }
    }

    #[test]
    fn test_open_order_exposure_limits_capacity_by_order_id() {
        let mut risk = RiskManager::new(&config());
        assert!(risk.can_place_orders("m1", &[120.0, 120.0]));

        risk.reserve_open_order("order-a", "m1", OpenOrderSide::Buy, 0.6, 200.0);
        risk.reserve_open_order("order-b", "m1", OpenOrderSide::Sell, 0.6, 200.0);
        assert_eq!(risk.open_order_count(), 2);
        assert!(!risk.can_place_orders("m1", &[120.0, 120.0]));

        risk.release_open_order("order-a");
        assert_eq!(risk.open_order_count(), 1);
        assert!(risk.can_place_orders("m1", &[120.0]));
    }

    #[test]
    fn test_apply_fill_reduces_exact_order_reservation() {
        let mut risk = RiskManager::new(&config());
        risk.reserve_open_order("order-a", "m1", OpenOrderSide::Buy, 0.5, 100.0);

        let effect = risk
            .apply_fill("order-a", 40.0, 0.5)
            .expect("fill should apply");

        assert!((effect.fill_notional_usd - 20.0).abs() < EPSILON);
        assert!(!effect.order_completed);
        assert!((effect.remaining_open_notional_usd - 30.0).abs() < EPSILON);
        assert!((risk.market_open_order_value("m1") - 30.0).abs() < EPSILON);
    }

    #[test]
    fn test_apply_fill_updates_inventory_and_realized_pnl() {
        let mut risk = RiskManager::new(&config());
        risk.reserve_open_order("buy-1", "m1", OpenOrderSide::Buy, 0.50, 100.0);
        let buy = risk
            .apply_fill("buy-1", 100.0, 0.50)
            .expect("buy fill should apply");
        assert!(buy.order_completed);
        assert_eq!(buy.net_position_shares, 100.0);

        risk.reserve_open_order("sell-1", "m1", OpenOrderSide::Sell, 0.60, 40.0);
        let sell = risk
            .apply_fill("sell-1", 40.0, 0.60)
            .expect("sell fill should apply");

        assert!(sell.order_completed);
        assert!((sell.realized_pnl_delta - 4.0).abs() < EPSILON);
        assert!((risk.daily_pnl() - 4.0).abs() < EPSILON);
        assert!((risk.market_position_value("m1") - 30.0).abs() < EPSILON);
        assert!((sell.net_position_shares - 60.0).abs() < EPSILON);
    }

    #[test]
    fn test_inventory_skew_signal_uses_net_position_notional() {
        let mut risk = RiskManager::new(&config());
        risk.reserve_open_order("buy-1", "m1", OpenOrderSide::Buy, 0.50, 70.0);
        risk.apply_fill("buy-1", 70.0, 0.50);

        let signal = risk.inventory_skew_signal("m1");
        assert!(signal > 0.0);
    }

    #[test]
    fn test_market_open_order_value_sums_exact_orders() {
        let mut risk = RiskManager::new(&config());
        risk.reserve_open_order("order-a", "m1", OpenOrderSide::Buy, 0.5, 200.0);
        risk.reserve_open_order("order-b", "m1", OpenOrderSide::Sell, 0.4, 200.0);
        risk.reserve_open_order("order-c", "m2", OpenOrderSide::Buy, 0.4, 100.0);

        assert!((risk.market_open_order_value("m1") - 180.0).abs() < EPSILON);
        assert!((risk.total_open_order_value() - 220.0).abs() < EPSILON);
    }
}
