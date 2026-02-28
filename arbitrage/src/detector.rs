//! 套利机会检测器

use polymarket_client_sdk::types::Decimal;
use rust_decimal_macros::dec;

use crate::config::StrategyConfig;
use crate::scanner::MarketPrice;

#[derive(Debug)]
pub enum ArbOpportunity {
    /// 买入套利：Yes + No < $1
    BuyAndMint {
        market_id: String,
        token_id_yes: String,
        token_id_no: String,
        profit_per_share: Decimal,
        max_shares: Decimal,
    },
    /// 卖出套利：Yes + No > $1
    RedeemAndSell {
        market_id: String,
        token_id_yes: String,
        token_id_no: String,
        profit_per_share: Decimal,
        max_shares: Decimal,
    },
}

pub struct Detector {
    min_profit: Decimal,
    max_position: Decimal,
    scan_interval_ms: u64,
}

impl Detector {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            min_profit: config.min_profit(),
            max_position: config.max_position(),
            scan_interval_ms: config.scan_interval_ms(),
        }
    }

    pub fn detect(&self, prices: &[MarketPrice]) -> Option<ArbOpportunity> {
        for market in prices {
            if market.volume_24h <= Decimal::ZERO {
                continue;
            }
            let sum = market.yes_price + market.no_price;

            if sum <= dec!(0.01) {
                continue;
            }

            // 买入套利：Yes + No < $1
            if sum < dec!(1) - self.min_profit {
                let profit_per_share = dec!(1) - sum;
                let max_shares = (self.max_position / sum).min(dec!(1000));

                return Some(ArbOpportunity::BuyAndMint {
                    market_id: market.market_id.clone(),
                    token_id_yes: market.token_id_yes.clone(),
                    token_id_no: market.token_id_no.clone(),
                    profit_per_share,
                    max_shares,
                });
            }

            // 卖出套利：Yes + No > $1
            if sum > dec!(1) + self.min_profit {
                let profit_per_share = sum - dec!(1);
                let max_shares = self.max_position.min(dec!(1000));

                return Some(ArbOpportunity::RedeemAndSell {
                    market_id: market.market_id.clone(),
                    token_id_yes: market.token_id_yes.clone(),
                    token_id_no: market.token_id_no.clone(),
                    profit_per_share,
                    max_shares,
                });
            }
        }
        None
    }

    pub fn scan_interval_ms(&self) -> u64 {
        self.scan_interval_ms
    }
}

impl std::fmt::Display for ArbOpportunity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArbOpportunity::BuyAndMint {
                profit_per_share,
                max_shares,
                ..
            } => {
                write!(
                    f,
                    "BuyAndMint: profit/share=${} shares={} total=${}",
                    profit_per_share,
                    max_shares,
                    profit_per_share * max_shares
                )
            }
            ArbOpportunity::RedeemAndSell {
                profit_per_share,
                max_shares,
                ..
            } => {
                write!(
                    f,
                    "RedeemAndSell: profit/share=${} shares={} total=${}",
                    profit_per_share,
                    max_shares,
                    profit_per_share * max_shares
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_detector() -> Detector {
        let config = StrategyConfig {
            min_profit_usd: 0.02,
            max_position_per_trade: 1000.0,
            scan_interval_ms: 50,
            gas_price_gwei: 50,
            include_all: true,
            exclude_market_ids: vec![],
        };
        Detector::new(&config)
    }

    fn create_test_market(yes: f64, no: f64) -> MarketPrice {
        MarketPrice {
            market_id: "test_market".to_string(),
            token_id_yes: "yes_token".to_string(),
            token_id_no: "no_token".to_string(),
            yes_price: Decimal::from_f64_retain(yes).unwrap(),
            no_price: Decimal::from_f64_retain(no).unwrap(),
            volume_24h: Decimal::from_f64_retain(10000.0).unwrap(),
        }
    }

    #[test]
    fn test_detect_buy_arbitrage() {
        let detector = create_test_detector();
        let prices = vec![
            create_test_market(0.40, 0.45), // sum = 0.85 < 1.0
        ];

        let opportunity = detector.detect(&prices);

        assert!(opportunity.is_some());
        match opportunity.unwrap() {
            ArbOpportunity::BuyAndMint {
                profit_per_share, ..
            } => {
                assert!(profit_per_share > Decimal::ZERO);
            }
            _ => panic!("Expected BuyAndMint opportunity"),
        }
    }

    #[test]
    fn test_detect_sell_arbitrage() {
        let detector = create_test_detector();
        let prices = vec![
            create_test_market(0.60, 0.55), // sum = 1.15 > 1.0
        ];

        let opportunity = detector.detect(&prices);

        assert!(opportunity.is_some());
        match opportunity.unwrap() {
            ArbOpportunity::RedeemAndSell {
                profit_per_share, ..
            } => {
                assert!(profit_per_share > Decimal::ZERO);
            }
            _ => panic!("Expected RedeemAndSell opportunity"),
        }
    }

    #[test]
    fn test_no_arbitrage_opportunity() {
        let detector = create_test_detector();
        let prices = vec![
            create_test_market(0.50, 0.49), // sum = 0.99, within threshold
        ];

        let opportunity = detector.detect(&prices);

        assert!(opportunity.is_none());
    }

    #[test]
    fn test_invalid_prices() {
        let detector = create_test_detector();
        let prices = vec![
            create_test_market(0.0, 0.0), // invalid prices
        ];

        let opportunity = detector.detect(&prices);

        assert!(opportunity.is_none());
    }

    #[test]
    fn test_profit_calculation() {
        let detector = create_test_detector();
        let prices = vec![
            create_test_market(0.40, 0.40), // sum = 0.80, profit = 0.20
        ];

        let opportunity = detector.detect(&prices);

        assert!(opportunity.is_some());
        match opportunity.unwrap() {
            ArbOpportunity::BuyAndMint {
                profit_per_share,
                max_shares,
                ..
            } => {
                let expected_profit = Decimal::from_f64_retain(0.20).unwrap();
                assert!(
                    (profit_per_share - expected_profit).abs()
                        < Decimal::from_f64_retain(0.01).unwrap()
                );
                assert!(max_shares > Decimal::ZERO);
            }
            _ => panic!("Expected BuyAndMint opportunity"),
        }
    }
}
