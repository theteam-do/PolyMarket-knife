//! 报价引擎

use polymarket_client_sdk::types::Decimal;
use rust_decimal_macros::dec;

use crate::config::StrategyConfig;

pub struct Quoter {
    spread_bps: u32,
    order_size: Decimal,
    min_spread_bps: u32,
    max_spread_bps: u32,
}

impl Quoter {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            spread_bps: config.spread_bps,
            order_size: Decimal::from_f64_retain(config.order_size_usd).unwrap_or(dec!(1000)),
            min_spread_bps: config.min_spread_bps,
            max_spread_bps: config.max_spread_bps,
        }
    }

    pub fn calculate_quotes(&self, mid_price: f64) -> (f64, f64) {
        let mid = match Decimal::from_f64_retain(mid_price) {
            Some(d) => d,
            None => return (0.0, 0.0),
        };

        let (bid, ask) = self.calculate_quotes_decimal(&mid);
        
        (bid.to_string().parse().unwrap_or(0.0), 
         ask.to_string().parse().unwrap_or(0.0))
    }

    pub fn calculate_quotes_decimal(&self, mid_price: &Decimal) -> (Decimal, Decimal) {
        if *mid_price <= dec!(0) {
            return (dec!(0), dec!(0));
        }

        // 计算有效价差
        let mut effective_spread = self.spread_bps;
        effective_spread = effective_spread.clamp(self.min_spread_bps, self.max_spread_bps);

        let spread_decimal = Decimal::from(effective_spread) / Decimal::from(10000);
        let half_spread = mid_price * &spread_decimal / dec!(2);
        
        let bid = mid_price - &half_spread;
        let ask = mid_price + &half_spread;

        // 确保价格在合理范围内 (0.01 - 0.99)
        let bid = bid.max(dec!(0.01)).min(dec!(0.99));
        let ask = ask.max(dec!(0.01)).min(dec!(0.99));
        
        // 确保 bid < ask
        let (bid, ask) = if bid >= ask {
            let mid = (bid + ask) / dec!(2);
            let half = dec!(0.01);
            ((mid - half).max(dec!(0.01)), (mid + half).min(dec!(0.99)))
        } else {
            (bid, ask)
        };

        (bid, ask)
    }

    pub fn order_size(&self) -> Decimal {
        self.order_size
    }
}
