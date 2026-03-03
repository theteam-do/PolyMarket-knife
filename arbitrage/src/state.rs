use crate::scanner::MarketPrice;
use polymarket_client_sdk::clob::ws::types::response::BookUpdate;
use polymarket_client_sdk::types::U256;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::debug;

pub struct MarketState {
    pub markets: HashMap<String, MarketPrice>,
    pub asset_to_market: HashMap<U256, (String, bool)>,
}

impl MarketState {
    pub fn new(initial_markets: Vec<MarketPrice>) -> Self {
        let mut markets = HashMap::new();
        let mut asset_to_market = HashMap::new();

        for m in initial_markets {
            if let Ok(yes_id) = U256::from_str(&m.token_id_yes) {
                asset_to_market.insert(yes_id, (m.market_id.clone(), true));
            }
            if let Ok(no_id) = U256::from_str(&m.token_id_no) {
                asset_to_market.insert(no_id, (m.market_id.clone(), false));
            }
            markets.insert(m.market_id.clone(), m);
        }

        Self {
            markets,
            asset_to_market,
        }
    }

    pub fn get_all_assets(&self) -> Vec<U256> {
        self.asset_to_market.keys().cloned().collect()
    }

    pub fn get_all_markets(&self) -> Vec<MarketPrice> {
        self.markets.values().cloned().collect()
    }

    fn update_price(&mut self, asset_id: &U256, price: Decimal) -> bool {
        if let Some((market_id, is_yes)) = self.asset_to_market.get(asset_id) {
            if let Some(market) = self.markets.get_mut(market_id) {
                if *is_yes {
                    market.yes_price = price;
                } else {
                    market.no_price = price;
                }
                debug!(
                    "Updated market {} asset {} price to {}",
                    market_id, asset_id, price
                );
                return true;
            }
        }
        false
    }

    /// 从订单簿更新中提取价格信息
    pub fn update_from_orderbook(&mut self, book: &BookUpdate) -> bool {
        let asset_id = &book.asset_id;
        let mut updated = false;

        // 使用最佳买卖价更新
        if let Some(best_bid) = book.bids.first() {
            if self.update_price(asset_id, best_bid.price) {
                updated = true;
            }
        }

        if let Some(best_ask) = book.asks.first() {
            // 对于 NO token，使用相反的逻辑
            if let Some((market_id, is_yes)) = self.asset_to_market.get(asset_id) {
                if let Some(market) = self.markets.get_mut(market_id) {
                    if *is_yes {
                        market.yes_price = best_ask.price;
                    } else {
                        market.no_price = best_ask.price;
                    }
                    debug!(
                        "Updated market {} asset {} ask price to {}",
                        market_id, asset_id, best_ask.price
                    );
                    updated = true;
                }
            }
        }

        updated
    }
}
