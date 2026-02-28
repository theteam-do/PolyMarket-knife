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
            ArbOpportunity::BuyAndMint { profit_per_share, max_shares, .. } => {
                write!(
                    f,
                    "BuyAndMint: profit/share=${} shares={} total=${}",
                    profit_per_share,
                    max_shares,
                    profit_per_share * max_shares
                )
            }
            ArbOpportunity::RedeemAndSell { profit_per_share, max_shares, .. } => {
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
