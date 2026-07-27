//! Market Maker 集成测试

use market_maker::config::{RiskConfig, SideMode, StrategyConfig};
use market_maker::order_book::{Level, OrderBook};
use market_maker::quoting::Quoter;
use market_maker::risk::{OpenOrderSide, RiskManager};
use rust_decimal_macros::dec;

#[test]
fn test_order_book_basic() {
    let mut book = OrderBook::new("token-1".to_string());
    book.bids = vec![Level {
        price: 0.49,
        size: 100.0,
    }];
    book.asks = vec![Level {
        price: 0.51,
        size: 120.0,
    }];
    book.update_best();

    assert_eq!(book.best_bid, Some(0.49));
    assert_eq!(book.best_ask, Some(0.51));
    assert_eq!(book.mid_price(), Some(0.50));
}

#[test]
fn test_risk_limits_track_exact_order_ids() {
    let mut risk = RiskManager::new(&RiskConfig {
        max_position_usd: 1_000.0,
        max_loss_per_day: 100.0,
        stop_loss_pct: 5.0,
        max_orders: 4,
        max_order_size_usd: 300.0,
    });

    assert!(risk.can_place_orders("market-1", &[120.0, 120.0]));
    risk.reserve_open_order("buy-1", "market-1", OpenOrderSide::Buy, 0.50, 240.0);
    risk.reserve_open_order("sell-1", "market-1", OpenOrderSide::Sell, 0.52, 250.0);

    assert!((risk.market_open_order_value("market-1") - 250.0).abs() < 1e-9);
    assert!(!risk.can_place_orders("market-1", &[120.0, 120.0]));

    risk.release_open_order("buy-1");
    assert!((risk.market_open_order_value("market-1") - 130.0).abs() < 1e-9);
}

#[test]
fn test_quote_calculation() {
    let quoter = Quoter::new(&StrategyConfig {
        market_ids: vec![],
        spread_bps: 100,
        order_size_usd: 1000.0,
        refresh_interval_ms: 100,
        skew_inventory: false,
        min_spread_bps: 50,
        max_spread_bps: 200,
        side_mode: SideMode::TwoSided,
    });

    let (bid, ask) = quoter.calculate_quotes_with_position(0.50, 0.0);
    assert!((bid - 0.4975).abs() < 0.0001);
    assert!((ask - 0.5025).abs() < 0.0001);
    assert!((ask - bid - 0.005).abs() < 0.0001);
}

#[test]
fn test_decimal_precision() {
    let price1 = dec!(0.50123);
    let price2 = dec!(0.49877);

    let sum = price1 + price2;
    assert!((sum - dec!(1.00000)).abs() < dec!(0.00001));

    let diff = price1 - price2;
    assert!((diff - dec!(0.00246)).abs() < dec!(0.00001));
}

#[test]
fn test_inventory_skew_biases_quotes() {
    let quoter = Quoter::new(&StrategyConfig {
        market_ids: vec![],
        spread_bps: 100,
        order_size_usd: 1000.0,
        refresh_interval_ms: 100,
        skew_inventory: true,
        min_spread_bps: 50,
        max_spread_bps: 200,
        side_mode: SideMode::TwoSided,
    });

    let (_plain_bid, plain_ask) = quoter.calculate_quotes_with_position(0.50, 0.0);
    let (_skewed_bid, skewed_ask) = quoter.calculate_quotes_with_position(0.50, 1.0);

    assert!(skewed_ask < plain_ask);
}
