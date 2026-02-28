//! 信号生成器

use std::collections::VecDeque;

use crate::config::StrategyConfig;
use crate::PriceTick;

#[derive(Debug, Clone)]
pub enum Signal {
    Buy { symbol: String, confidence: f64 },
    Sell { symbol: String, confidence: f64 },
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

        let variance = prices.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / prices.len() as f64;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_generator() -> SignalGenerator {
        let config = StrategyConfig {
            symbols: vec!["BTCUSDT".to_string()],
            volatility_threshold: 0.02,
            momentum_threshold: 0.01,
            base_position_usd: 100.0,
            max_position_usd: 10000.0,
            confidence_high: 0.8,
            max_loss_per_trade: 100.0,
            max_daily_loss: 500.0,
            stop_loss_pct: 0.1,
        };
        SignalGenerator::new(&config)
    }

    fn create_test_tick(price: f64, volume: f64) -> PriceTick {
        PriceTick {
            symbol: "BTCUSDT".to_string(),
            price,
            timestamp: 1234567890,
            volume,
        }
    }

    #[test]
    fn test_no_signal_with_insufficient_data() {
        let mut gen = create_test_generator();

        // Only 5 data points (need 10)
        for i in 0..5 {
            let tick = create_test_tick(50000.0 + i as f64, 1.0);
            assert!(gen.generate(&tick).is_none());
        }
    }

    #[test]
    fn test_volatility_calculation() {
        let mut gen = create_test_generator();

        // High volatility prices
        let prices = vec![
            50000.0, 51000.0, 49000.0, 52000.0, 48000.0, 53000.0, 47000.0, 54000.0, 46000.0,
            55000.0,
        ];
        for (i, &price) in prices.iter().enumerate() {
            let tick = create_test_tick(price, 10.0);
            if i == prices.len() - 1 {
                let signal = gen.generate(&tick);
                // High volatility should generate signal
                assert!(signal.is_some());
            } else {
                gen.generate(&tick);
            }
        }
    }

    #[test]
    fn test_confidence_range() {
        let mut gen = create_test_generator();

        // Create enough data
        for i in 0..15 {
            let price = 50000.0 * (1.0 + (i as f64 * 0.01));
            let tick = create_test_tick(price, 10.0);

            if i == 14 {
                if let Some(signal) = gen.generate(&tick) {
                    assert!(signal.confidence() >= 0.0);
                    assert!(signal.confidence() <= 1.0);
                }
            } else {
                gen.generate(&tick);
            }
        }
    }

    #[test]
    fn test_momentum_positive() {
        let mut gen = create_test_generator();

        // Upward trend
        for i in 0..15 {
            let price = 50000.0 * (1.0 + i as f64 * 0.005);
            let tick = create_test_tick(price, 10.0);

            if i == 14 {
                if let Some(signal) = gen.generate(&tick) {
                    match signal {
                        Signal::Buy { .. } => {} // Expected
                        Signal::Sell { .. } => panic!("Expected Buy signal for upward trend"),
                    }
                }
            } else {
                gen.generate(&tick);
            }
        }
    }

    #[test]
    fn test_momentum_negative() {
        let mut gen = create_test_generator();

        // Downward trend
        for i in 0..15 {
            let price = 50000.0 * (1.0 - i as f64 * 0.005);
            let tick = create_test_tick(price, 10.0);

            if i == 14 {
                if let Some(signal) = gen.generate(&tick) {
                    match signal {
                        Signal::Sell { .. } => {} // Expected
                        Signal::Buy { .. } => panic!("Expected Sell signal for downward trend"),
                    }
                }
            } else {
                gen.generate(&tick);
            }
        }
    }
}
