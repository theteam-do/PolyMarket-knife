use crate::scanner::MarketPrice;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::debug;
use std::str::FromStr;
use serde_json::Value;

pub struct MarketState {
    pub markets: HashMap<String, MarketPrice>,
    pub asset_to_market: HashMap<String, (String, bool)>, 
}

impl MarketState {
    pub fn new(initial_markets: Vec<MarketPrice>) -> Self {
        let mut markets = HashMap::new();
        let mut asset_to_market = HashMap::new();

        for m in initial_markets {
            asset_to_market.insert(m.token_id_yes.clone(), (m.market_id.clone(), true));
            asset_to_market.insert(m.token_id_no.clone(), (m.market_id.clone(), false));
            markets.insert(m.market_id.clone(), m);
        }

        Self {
            markets,
            asset_to_market,
        }
    }

    pub fn get_all_assets(&self) -> Vec<String> {
        self.asset_to_market.keys().cloned().collect()
    }

    pub fn get_all_markets(&self) -> Vec<MarketPrice> {
        self.markets.values().cloned().collect()
    }

    pub fn update_from_ws_payload(&mut self, event_type: &str, payload: &Value) -> bool {
        let mut updated = false;
        if event_type == "price_change" {
            if let Some(changes) = payload.get("price_changes").and_then(|c| c.as_array()) {
                for change in changes {
                    if let Some(asset_id) = change.get("asset_id").and_then(|a| a.as_str()) {
                        let price_str = change.get("price").and_then(|p| p.as_str());
                        if let Some(price_str) = price_str {
                            if let Ok(price) = Decimal::from_str(price_str) {
                                if self.update_price(asset_id, price) {
                                    updated = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        updated
    }

    fn update_price(&mut self, asset_id: &str, price: Decimal) -> bool {
        if let Some((market_id, is_yes)) = self.asset_to_market.get(asset_id) {
            if let Some(market) = self.markets.get_mut(market_id) {
                if *is_yes {
                    market.yes_price = price;
                } else {
                    market.no_price = price;
                }
                debug!("Updated market {} asset {} price to {}", market_id, asset_id, price);
                return true;
            }
        }
        false
    }
}
