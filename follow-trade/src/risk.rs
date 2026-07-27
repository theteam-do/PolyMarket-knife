use crate::config::Config;

#[derive(Default)]
pub struct RiskManager {
    pub daily_pnl: f64,
    pub total_exposure: f64,
}

impl RiskManager {
    pub fn can_trade(&self, config: &Config) -> bool {
        self.daily_pnl >= -config.strategy.max_daily_loss
            && self.total_exposure <= config.strategy.max_position_per_market
    }

    pub fn update_exposure(&mut self, notional: f64) {
        self.total_exposure += notional;
    }

    pub fn update_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
    }

    pub fn total_position_value(&self) -> f64 {
        self.total_exposure
    }

    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
    }
}
