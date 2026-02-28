//! 套利机会检测器

use crate::config::StrategyConfig;
use crate::scanner::MarketPrice;
use rust_decimal::Decimal;

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
}

impl Detector {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            min_profit: Decimal::from_f64_retain(config.min_profit_usd)
                .unwrap_or(Decimal::from_f64_retain(0.02).unwrap()),
            max_position: Decimal::from_f64_retain(config.max_position_per_trade)
                .unwrap_or(Decimal::from(1000)),
        }
    }

    pub fn detect(&self, prices: &[MarketPrice]) -> Option<ArbOpportunity> {
        for market in prices {
            let sum = market.yes_price + market.no_price;

            if sum <= Decimal::from_f64_retain(0.01).unwrap() {
                continue;
            }

            // 买入套利：Yes + No < $1
            if sum < Decimal::ONE - self.min_profit {
                let profit_per_share = Decimal::ONE - sum;
                let max_shares = (self.max_position / sum).min(Decimal::from(1000));

                return Some(ArbOpportunity::BuyAndMint {
                    market_id: market.market_id.clone(),
                    token_id_yes: market.token_id_yes.clone(),
                    token_id_no: market.token_id_no.clone(),
                    profit_per_share,
                    max_shares,
                });
            }

            // 卖出套利：Yes + No > $1
            if sum > Decimal::ONE + self.min_profit {
                let profit_per_share = sum - Decimal::ONE;
                let max_shares = self.max_position.min(Decimal::from(1000));

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
                    "BuyAndMint: profit/share=${}, max_shares={}, total_profit=${}",
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
                    "RedeemAndSell: profit/share=${}, max_shares={}, total_profit=${}",
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
            volume_24h: Decimal::from(10000),
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
