//! 信号生成器

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::VecDeque;

use crate::config::StrategyConfig;
use crate::PriceTick;

#[derive(Debug, Clone)]
pub enum Signal {
    Buy {
        symbol: String,
        confidence: f64,
    },
    Sell {
        symbol: String,
        confidence: f64,
    },
}

impl Signal {
    pub fn confidence(&self) -> f64 {
        match self {
            Signal::Buy { confidence, .. } | Signal::Sell { confidence, .. } => *confidence,
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            Signal::Buy { symbol, .. } | Signal::Sell { symbol, .. } => symbol,
        }
    }
}

pub struct SignalGenerator {
    config: StrategyConfig,
    price_history: VecDeque<f64>,
    volume_history: VecDeque<f64>,
    window_size: usize,
}

impl SignalGenerator {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
            price_history: VecDeque::with_capacity(100),
            volume_history: VecDeque::with_capacity(100),
            window_size: 50,
        }
    }

    pub fn generate(&mut self, tick: &PriceTick) -> Option<Signal> {
        self.price_history.push_back(tick.price);
        self.volume_history.push_back(tick.volume);
        
        while self.price_history.len() > self.window_size {
            self.price_history.pop_front();
            self.volume_history.pop_front();
        }

        if self.price_history.len() < 10 {
            return None;
        }

        let volatility = self.calc_volatility();
        let momentum = self.calc_momentum();

        if volatility > self.config.volatility_threshold {
            let confidence = self.calc_confidence(volatility, momentum);
            
            if momentum > self.config.momentum_threshold {
                return Some(Signal::Buy {
                    symbol: tick.symbol.clone(),
                    confidence,
                });
            } else if momentum < -self.config.momentum_threshold {
                return Some(Signal::Sell {
                    symbol: tick.symbol.clone(),
                    confidence,
                });
            }
        }

        None
    }

    fn calc_volatility(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }

        let prices: Vec<f64> = self.price_history.iter().copied().collect();
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        
        let variance = prices.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        
        variance.sqrt() / mean
    }

    fn calc_momentum(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }

        let prices: Vec<f64> = self.price_history.iter().copied().collect();
        let latest = prices.last().unwrap();
        let previous = prices[prices.len() - 2];
        
        (latest - previous) / previous
    }

    fn calc_confidence(&self, volatility: f64, momentum: f64) -> f64 {
        let mut confidence = 0.5;
        confidence += (volatility / 0.05).min(0.3);
        confidence += (momentum.abs() / 0.02).min(0.2);
        confidence.min(1.0)
    }
}
