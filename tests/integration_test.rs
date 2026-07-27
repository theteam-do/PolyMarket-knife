use rust_decimal_macros::dec;

use common::{
    arbitrage_gross_edge, arbitrage_gross_edge_per_share, bayes_update,
    capped_notional, clamp_probability, fractional_kelly,
    implied_probability_from_price, kelly_fraction, ExecutionConfig, ExecutionMode,
};

#[test]
fn test_enforce_safety_paper_default() {
    let config = ExecutionConfig::default();
    assert_eq!(config.mode, ExecutionMode::Paper);
    assert!(config.enforce_safety().is_ok());
}

#[test]
fn test_enforce_safety_live_unacknowledged_fails() {
    let config = ExecutionConfig {
        mode: ExecutionMode::Live,
        live_acknowledged: false,
        ..Default::default()
    };
    assert!(config.enforce_safety().is_err());
}

#[test]
fn test_enforce_safety_live_acknowledged_passes() {
    let config = ExecutionConfig {
        mode: ExecutionMode::Live,
        live_acknowledged: true,
        ..Default::default()
    };
    assert!(config.enforce_safety().is_ok());
}

#[test]
fn test_enforce_safety_live_no_ack_required() {
    let config = ExecutionConfig {
        mode: ExecutionMode::Live,
        require_explicit_live_ack: false,
        live_acknowledged: false,
        ..Default::default()
    };
    assert!(config.enforce_safety().is_ok());
}

#[test]
fn test_kelly_fraction_positive_edge() {
    let f = kelly_fraction(dec!(0.55), dec!(1)).unwrap();
    assert!(f > dec!(0));
    let expected = (dec!(1) * dec!(0.55) - dec!(0.45)) / dec!(1);
    assert!((f - expected).abs() < dec!(0.0001));
}

#[test]
fn test_kelly_fraction_negative_edge_returns_zero() {
    let f = kelly_fraction(dec!(0.40), dec!(1)).unwrap();
    assert_eq!(f, dec!(0));
}

#[test]
fn test_kelly_fraction_rejects_non_positive_odds() {
    assert!(kelly_fraction(dec!(0.5), dec!(0)).is_err());
    assert!(kelly_fraction(dec!(0.5), dec!(-1)).is_err());
}

#[test]
fn test_fractional_kelly_scales_down() {
    let full = kelly_fraction(dec!(0.6), dec!(1)).unwrap();
    let half = fractional_kelly(dec!(0.6), dec!(1), dec!(0.5)).unwrap();
    assert!((half - full * dec!(0.5)).abs() < dec!(0.0001));
}

#[test]
fn test_fractional_kelly_rejects_out_of_range() {
    assert!(fractional_kelly(dec!(0.5), dec!(1), dec!(0)).is_err());
    assert!(fractional_kelly(dec!(0.5), dec!(1), dec!(2)).is_err());
}

#[test]
fn test_capped_notional_respects_max() {
    let result = capped_notional(dec!(10000), dec!(0.5), dec!(1000)).unwrap();
    assert_eq!(result, dec!(1000));
}

#[test]
fn test_capped_notional_below_max() {
    let result = capped_notional(dec!(1000), dec!(0.5), dec!(10000)).unwrap();
    assert_eq!(result, dec!(500));
}

#[test]
fn test_capped_notional_rejects_invalid() {
    assert!(capped_notional(dec!(0), dec!(0.1), dec!(10)).is_err());
    assert!(capped_notional(dec!(100), dec!(-1), dec!(10)).is_err());
    assert!(capped_notional(dec!(100), dec!(0.1), dec!(0)).is_err());
}

#[test]
fn test_clamp_probability_lower_bound() {
    assert_eq!(clamp_probability(dec!(0)), dec!(0.000001));
    assert_eq!(clamp_probability(dec!(-1)), dec!(0.000001));
}

#[test]
fn test_clamp_probability_upper_bound() {
    assert_eq!(clamp_probability(dec!(1)), dec!(0.999999));
    assert_eq!(clamp_probability(dec!(2)), dec!(0.999999));
}

#[test]
fn test_clamp_probability_mid_range() {
    assert_eq!(clamp_probability(dec!(0.5)), dec!(0.5));
}

#[test]
fn test_implied_probability_valid() {
    let p = implied_probability_from_price(dec!(0.65)).unwrap();
    assert_eq!(p, dec!(0.65));
}

#[test]
fn test_implied_probability_rejects_out_of_range() {
    assert!(implied_probability_from_price(dec!(-0.01)).is_err());
    assert!(implied_probability_from_price(dec!(1.01)).is_err());
}

#[test]
fn test_bayes_update_posterior_higher_with_good_likelihood() {
    let posterior = bayes_update(dec!(0.5), dec!(0.9), dec!(0.1)).unwrap();
    assert!(posterior > dec!(0.5));
    assert!(posterior < dec!(1));
}

#[test]
fn test_bayes_update_rejects_invalid_inputs() {
    assert!(bayes_update(dec!(1.5), dec!(0.7), dec!(0.3)).is_err());
}

#[test]
fn test_arbitrage_gross_edge_per_share_undervalued() {
    let edge = arbitrage_gross_edge_per_share(dec!(0.92), dec!(1)).unwrap();
    assert_eq!(edge, dec!(0.08));
}

#[test]
fn test_arbitrage_gross_edge_per_share_overvalued() {
    let edge = arbitrage_gross_edge_per_share(dec!(1.08), dec!(1)).unwrap();
    assert_eq!(edge, dec!(0.08));
}

#[test]
fn test_arbitrage_gross_edge_per_share_no_edge() {
    let edge = arbitrage_gross_edge_per_share(dec!(1), dec!(1)).unwrap();
    assert_eq!(edge, dec!(0));
}

#[test]
fn test_arbitrage_gross_edge_per_share_rejects_invalid() {
    assert!(arbitrage_gross_edge_per_share(dec!(0), dec!(1)).is_err());
    assert!(arbitrage_gross_edge_per_share(dec!(0.95), dec!(0)).is_err());
}

#[test]
fn test_arbitrage_gross_edge_scales_with_shares() {
    let per_share = arbitrage_gross_edge_per_share(dec!(0.95), dec!(1)).unwrap();
    let total = arbitrage_gross_edge(dec!(0.95), dec!(1), dec!(100)).unwrap();
    assert_eq!(total, per_share * dec!(100));
}

#[test]
fn test_arbitrage_gross_edge_rejects_non_positive_shares() {
    assert!(arbitrage_gross_edge(dec!(0.95), dec!(1), dec!(0)).is_err());
}

// ---------------------------------------------------------------------------
// monitor crate: AlertManager, Dashboard, Metrics
// ---------------------------------------------------------------------------

use monitor::{
    alerts::{AlertConfig, AlertManager, Severity},
    dashboard::Dashboard,
    metrics::{Metrics, OrderStatus, Timer},
};

#[test]
fn test_alert_config_defaults() {
    let config = AlertConfig::default();
    assert_eq!(config.daily_loss_threshold, dec!(500));
    assert_eq!(config.daily_loss_warning_pct, 0.8);
    assert_eq!(config.position_threshold, dec!(10000));
    assert_eq!(config.latency_threshold_ms, 100.0);
    assert_eq!(config.consecutive_loss_threshold, 5);
}

#[test]
fn test_alert_daily_loss_no_alert_when_below_warning() {
    let mut mgr = AlertManager::default();
    let alerts = mgr.check_daily_loss(dec!(-100));
    assert!(alerts.is_empty());
}

#[test]
fn test_alert_daily_loss_warning_at_80_percent() {
    let mut mgr = AlertManager::default();
    let alerts = mgr.check_daily_loss(dec!(-450));
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, Severity::Warning);
}

#[test]
fn test_alert_daily_loss_critical_when_exceeded() {
    let mut mgr = AlertManager::default();
    let alerts = mgr.check_daily_loss(dec!(-550));
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, Severity::Critical);
}

#[test]
fn test_alert_daily_loss_cooldown_suppresses_duplicates() {
    let mut mgr = AlertManager::default();
    let first = mgr.check_daily_loss(dec!(-550));
    assert_eq!(first.len(), 1);
    let second = mgr.check_daily_loss(dec!(-600));
    assert!(second.is_empty());
}

#[test]
fn test_alert_should_stop_trading() {
    let mut mgr = AlertManager::default();
    assert!(!mgr.should_stop_trading());
    mgr.check_daily_loss(dec!(-550));
    assert!(mgr.should_stop_trading());
}

#[test]
fn test_alert_position_exceeded() {
    let mut mgr = AlertManager::default();
    let alerts = mgr.check_position(dec!(15000));
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, Severity::Critical);
}

#[test]
fn test_alert_position_within_limit() {
    let mut mgr = AlertManager::default();
    let alerts = mgr.check_position(dec!(5000));
    assert!(alerts.is_empty());
}

#[test]
fn test_alert_latency_exceeded() {
    let mut mgr = AlertManager::default();
    let alerts = mgr.check_latency(200.0);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, Severity::Warning);
}

#[test]
fn test_alert_latency_within_limit() {
    let mut mgr = AlertManager::default();
    let alerts = mgr.check_latency(50.0);
    assert!(alerts.is_empty());
}

#[test]
fn test_alert_consecutive_losses_triggers_at_threshold() {
    let mut mgr = AlertManager::default();
    assert!(mgr.check_consecutive_losses(4).is_empty());
    let alerts = mgr.check_consecutive_losses(5);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, Severity::Critical);
}

#[test]
fn test_alert_order_failure_records_and_cooldown_works() {
    let mut mgr = AlertManager::default();
    let alerts = mgr.record_order_failure("ord-1", "insufficient liquidity");
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, Severity::Warning);
    let suppressed = mgr.record_order_failure("ord-2", "timeout");
    assert!(suppressed.is_empty());
}

#[test]
fn test_alert_clear_resets_history() {
    let mut mgr = AlertManager::default();
    mgr.check_daily_loss(dec!(-550));
    assert!(!mgr.get_alerts().is_empty());
    mgr.clear_alerts();
    assert!(mgr.get_alerts().is_empty());
}

#[test]
fn test_metrics_new_initializes_at_zero() {
    let m = Metrics::new();
    assert_eq!(m.daily_pnl.get(), 0.0);
    assert_eq!(m.orders_placed.get(), 0.0);
}

#[test]
fn test_metrics_record_order_increments_counters() {
    let m = Metrics::new();
    m.record_order(OrderStatus::Filled, 50.0);
    assert_eq!(m.orders_placed.get(), 1.0);
    assert_eq!(m.orders_filled.get(), 1.0);
}

#[test]
fn test_metrics_record_order_cancelled() {
    let m = Metrics::new();
    m.record_order(OrderStatus::Cancelled, 30.0);
    assert_eq!(m.orders_placed.get(), 1.0);
    assert_eq!(m.orders_cancelled.get(), 1.0);
}

#[test]
fn test_metrics_record_order_failed() {
    let m = Metrics::new();
    m.record_order(OrderStatus::Failed, 20.0);
    assert_eq!(m.orders_placed.get(), 1.0);
    assert_eq!(m.orders_failed.get(), 1.0);
}

#[test]
fn test_metrics_record_pnl_updates_gauge() {
    let m = Metrics::new();
    m.record_pnl(dec!(250.50));
    assert!(m.daily_pnl.get() > 250.0);
    assert!(m.daily_pnl.get() < 251.0);
}

#[test]
fn test_metrics_record_loss_updates_drawdown() {
    let m = Metrics::new();
    m.record_pnl(dec!(-75.0));
    assert!(m.daily_pnl.get() < -74.0);
    assert!(m.max_drawdown.get() > 74.0);
}

#[test]
fn test_metrics_timer_measures_elapsed() {
    let t = Timer::new();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let elapsed = t.elapsed_ms();
    assert!(elapsed >= 5.0 && elapsed < 500.0);
}

#[test]
fn test_metrics_gather_contains_labels() {
    let m = Metrics::new();
    m.orders_placed.inc();
    let output = m.gather();
    assert!(output.contains("orders_placed_total"));
}

#[test]
fn test_dashboard_aggregates_status() {
    let metrics = Metrics::new();
    let alert_mgr = AlertManager::default();
    let dash = Dashboard::new(metrics, alert_mgr);
    let status = dash.get_status();
    assert_eq!(status.daily_pnl, 0.0);
    assert_eq!(status.orders_placed, 0);
    assert!(!status.should_stop);
}

// ---------------------------------------------------------------------------
// market-maker crate: OrderBook, Quoter, RiskManager, SideMode
// ---------------------------------------------------------------------------

use market_maker::{
    config::{RiskConfig as MmRiskConfig, SideMode, StrategyConfig},
    order_book::{Level, OrderBook},
    quoting::Quoter,
    risk::{OpenOrderSide, RiskManager},
};

// --- OrderBook -------------------------------------------------------------

#[test]
fn test_order_book_new_creates_empty_book() {
    let book = OrderBook::new("0xabc".into());
    assert_eq!(book.token_id, "0xabc");
    assert!(book.bids.is_empty());
    assert!(book.asks.is_empty());
    assert!(book.best_bid.is_none());
    assert!(book.best_ask.is_none());
}

#[test]
fn test_order_book_mid_price_happy_path() {
    let mut book = OrderBook::new("t1".into());
    book.bids.push(Level { price: 0.50, size: 100.0 });
    book.asks.push(Level { price: 0.52, size: 100.0 });
    book.update_best();
    let mid = book.mid_price().unwrap();
    assert!((mid - 0.51).abs() < 0.0001);
}

#[test]
fn test_order_book_mid_price_returns_none_when_empty() {
    let book = OrderBook::new("t1".into());
    assert!(book.mid_price().is_none());
}

#[test]
fn test_order_book_spread_happy_path() {
    let mut book = OrderBook::new("t1".into());
    book.bids.push(Level { price: 0.50, size: 100.0 });
    book.asks.push(Level { price: 0.52, size: 100.0 });
    book.update_best();
    let spread = book.spread().unwrap();
    assert!((spread - 0.02).abs() < 0.0001);
}

#[test]
fn test_order_book_spread_bps() {
    let mut book = OrderBook::new("t1".into());
    // mid = 0.505, spread = 0.02 → ~396 bps
    book.bids.push(Level { price: 0.50, size: 100.0 });
    book.asks.push(Level { price: 0.52, size: 100.0 });
    book.update_best();
    let bps = book.spread_bps().unwrap();
    assert!(bps > 0);
    assert!(bps < 10_000);
}

#[test]
fn test_order_book_update_best_selects_top_level() {
    let mut book = OrderBook::new("t1".into());
    book.bids = vec![
        Level { price: 0.50, size: 100.0 },
        Level { price: 0.49, size: 200.0 },
    ];
    book.asks = vec![
        Level { price: 0.52, size: 100.0 },
        Level { price: 0.53, size: 200.0 },
    ];
    book.update_best();
    assert_eq!(book.best_bid, Some(0.50));
    assert_eq!(book.best_ask, Some(0.52));
}

#[test]
fn test_order_book_spread_returns_none_partial_book() {
    let mut book = OrderBook::new("t1".into());
    book.bids.push(Level { price: 0.50, size: 100.0 });
    book.update_best();
    assert!(book.spread().is_none());
    assert!(book.mid_price().is_none());
}

// --- Quoter ----------------------------------------------------------------

fn test_strategy_config(spread_bps: u32, skew: bool) -> StrategyConfig {
    StrategyConfig {
        market_ids: vec![],
        spread_bps,
        order_size_usd: 500.0,
        refresh_interval_ms: 100,
        skew_inventory: skew,
        min_spread_bps: 50,
        max_spread_bps: 500,
        side_mode: SideMode::TwoSided,
        metrics_bind_addr: "127.0.0.1:9090".to_string(),
    }
}

#[test]
fn test_quoter_calculates_bid_ask_from_mid() {
    let quoter = Quoter::new(&test_strategy_config(100, false));
    let (bid, ask) = quoter.calculate_quotes_with_position(0.50, 0.0);
    assert!(bid < ask);
    assert!(bid >= 0.01 && bid <= 0.99);
    assert!(ask >= 0.01 && ask <= 0.99);
}

#[test]
fn test_quoter_returns_zero_for_zero_price() {
    let quoter = Quoter::new(&test_strategy_config(100, false));
    let (bid, ask) = quoter.calculate_quotes_with_position(0.0, 0.0);
    assert_eq!(bid, 0.0);
    assert_eq!(ask, 0.0);
}

#[test]
fn test_quoter_skew_shifts_quotes() {
    let neutral = Quoter::new(&test_strategy_config(100, true));
    let (_b1, a1) = neutral.calculate_quotes_with_position(0.50, 0.0);
    let (_b2, a2) = neutral.calculate_quotes_with_position(0.50, 1.0);
    // Long skew → lower ask to encourage selling
    assert!(a2 <= a1 + 0.001);
}

#[test]
fn test_quoter_negative_skew_shifts_quotes_down() {
    let neutral = Quoter::new(&test_strategy_config(100, true));
    let (b1, _a1) = neutral.calculate_quotes_with_position(0.50, 0.0);
    let (b2, _a2) = neutral.calculate_quotes_with_position(0.50, -1.0);
    // Short skew → higher bid to encourage buying
    assert!(b2 >= b1 - 0.001);
}

#[test]
fn test_quoter_spread_stays_within_configured_bounds() {
    let quoter = Quoter::new(&test_strategy_config(100, false));
    let (bid, ask) = quoter.calculate_quotes_with_position(0.50, 0.0);
    let spread_bps = ((ask - bid) / ((bid + ask) / 2.0)) * 10_000.0;
    assert!(spread_bps >= 50.0, "spread below min");
    assert!(spread_bps <= 500.0, "spread above max");
}

#[test]
fn test_quoter_clamps_to_valid_range() {
    let quoter = Quoter::new(&test_strategy_config(100, false));
    let (bid, ask) = quoter.calculate_quotes_with_position(0.99, 0.0);
    assert!(bid < ask, "Bid should be less than ask");
    assert!(bid >= 0.001, "Bid should not exceed lower bound");
    assert!(ask <= 0.999, "Ask should not exceed upper bound");
}

#[test]
fn test_quoter_prevents_bid_ask_inversion() {
    let quoter = Quoter::new(&test_strategy_config(500, true));
    let (bid, ask) = quoter.calculate_quotes_with_position(0.50, -1.0);
    assert!(bid < ask, "bid must remain below ask");
}

// --- RiskManager -----------------------------------------------------------

fn test_risk_config() -> MmRiskConfig {
    MmRiskConfig {
        max_position_usd: 1_000.0,
        max_loss_per_day: 100.0,
        stop_loss_pct: 5.0,
        max_orders: 3,
        max_order_size_usd: 250.0,
        max_market_concentration_pct: 0.3,
    }
}

#[test]
fn test_risk_manager_can_trade_by_default() {
    let risk = RiskManager::new(&test_risk_config());
    assert!(risk.can_trade());
}

#[test]
fn test_risk_manager_cannot_trade_when_daily_loss_exceeded() {
    let mut risk = RiskManager::new(&test_risk_config());
    // Buy 500 @ 0.50 → cost 250
    risk.reserve_open_order("buy", "m1", OpenOrderSide::Buy, 0.50, 500.0);
    risk.apply_fill("buy", 500.0, 0.50);
    // Sell 500 @ 0.28 → realized PnL = (0.28 - 0.50) × 500 = -110 < -100 max loss
    risk.reserve_open_order("sell", "m1", OpenOrderSide::Sell, 0.28, 500.0);
    risk.apply_fill("sell", 500.0, 0.28);
    assert!(risk.daily_pnl() < -100.0);
    assert!(!risk.can_trade());
}

#[test]
fn test_risk_manager_can_place_orders_respects_max_orders() {
    let mut risk = RiskManager::new(&test_risk_config());
    assert!(risk.can_place_orders("m1", &[100.0]));
    risk.reserve_open_order("a", "m1", OpenOrderSide::Buy, 0.5, 200.0);
    risk.reserve_open_order("b", "m2", OpenOrderSide::Buy, 0.5, 200.0);
    risk.reserve_open_order("c", "m3", OpenOrderSide::Buy, 0.5, 200.0);
    assert_eq!(risk.open_order_count(), 3);
    assert!(!risk.can_place_orders("m4", &[100.0]));
}

#[test]
fn test_risk_manager_can_place_orders_rejects_oversized() {
    let risk = RiskManager::new(&test_risk_config());
    assert!(!risk.can_place_orders("m1", &[300.0]));
}

#[test]
fn test_risk_manager_can_place_orders_rejects_exceeding_position_limit() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.reserve_open_order("big", "m1", OpenOrderSide::Buy, 0.5, 1_800.0);
    // 1_800 shares × 0.50 = 900, plus 200 proposed = 1100 > 1000 max
    assert!(!risk.can_place_orders("m1", &[200.0]));
}

#[test]
fn test_risk_manager_reserve_and_release_open_order() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.reserve_open_order("o1", "m1", OpenOrderSide::Buy, 0.50, 100.0);
    assert_eq!(risk.open_order_count(), 1);
    let released = risk.release_open_order("o1");
    assert!(released.is_some());
    assert!((released.unwrap() - 50.0).abs() < 1e-6);
    assert_eq!(risk.open_order_count(), 0);
}

#[test]
fn test_risk_manager_apply_partial_fill() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.reserve_open_order("o1", "m1", OpenOrderSide::Buy, 0.50, 100.0);
    let effect = risk
        .apply_fill("o1", 40.0, 0.50)
        .expect("partial fill should succeed");
    assert!((effect.fill_notional_usd - 20.0).abs() < 1e-6);
    assert!(!effect.order_completed);
    assert!((effect.remaining_open_notional_usd - 30.0).abs() < 1e-6);
}

#[test]
fn test_risk_manager_apply_full_fill_completes_order() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.reserve_open_order("o1", "m1", OpenOrderSide::Buy, 0.50, 100.0);
    let effect = risk
        .apply_fill("o1", 100.0, 0.50)
        .expect("full fill should succeed");
    assert!(effect.order_completed);
    assert_eq!(effect.net_position_shares, 100.0);
    assert!((effect.remaining_open_notional_usd - 0.0).abs() < 1e-6);
}

#[test]
fn test_risk_manager_realized_pnl_from_profitable_sell() {
    let mut risk = RiskManager::new(&test_risk_config());
    // Buy 100 @ 0.50
    risk.reserve_open_order("buy", "m1", OpenOrderSide::Buy, 0.50, 100.0);
    risk.apply_fill("buy", 100.0, 0.50);
    // Sell 40 @ 0.60 → profit 0.10 × 40 = 4.0
    risk.reserve_open_order("sell", "m1", OpenOrderSide::Sell, 0.60, 40.0);
    let effect = risk
        .apply_fill("sell", 40.0, 0.60)
        .expect("sell fill should succeed");
    assert!((effect.realized_pnl_delta - 4.0).abs() < 1e-6);
    assert!((risk.daily_pnl() - 4.0).abs() < 1e-6);
}

#[test]
fn test_risk_manager_realized_pnl_from_loss_sell() {
    let mut risk = RiskManager::new(&test_risk_config());
    // Buy 100 @ 0.50
    risk.reserve_open_order("buy", "m1", OpenOrderSide::Buy, 0.50, 100.0);
    risk.apply_fill("buy", 100.0, 0.50);
    // Sell 50 @ 0.40 → loss 0.10 × 50 = -5.0
    risk.reserve_open_order("sell", "m1", OpenOrderSide::Sell, 0.40, 50.0);
    let effect = risk
        .apply_fill("sell", 50.0, 0.40)
        .expect("sell fill should succeed");
    assert!((effect.realized_pnl_delta - (-5.0)).abs() < 1e-6);
}

#[test]
fn test_risk_manager_inventory_skew_signal_positive_for_long() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.reserve_open_order("buy", "m1", OpenOrderSide::Buy, 0.50, 70.0);
    risk.apply_fill("buy", 70.0, 0.50);
    let signal = risk.inventory_skew_signal("m1");
    assert!(signal > 0.0);
}

#[test]
fn test_risk_manager_inventory_skew_signal_negative_for_short() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.reserve_open_order("sell", "m1", OpenOrderSide::Sell, 0.50, 70.0);
    risk.apply_fill("sell", 70.0, 0.50);
    let signal = risk.inventory_skew_signal("m1");
    assert!(signal < 0.0);
}

#[test]
fn test_risk_manager_inventory_skew_signal_zero_when_no_position() {
    let risk = RiskManager::new(&test_risk_config());
    assert_eq!(risk.inventory_skew_signal("m1"), 0.0);
}

#[test]
fn test_risk_manager_stop_and_resume_trading() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.stop_trading();
    assert!(!risk.can_trade());
    risk.resume_trading();
    assert!(risk.can_trade());
}

#[test]
fn test_risk_manager_reset_daily_clears_pnl() {
    let mut risk = RiskManager::new(&test_risk_config());
    // Generate non-zero PnL via a profitable round-trip
    risk.reserve_open_order("buy", "m1", OpenOrderSide::Buy, 0.50, 100.0);
    risk.apply_fill("buy", 100.0, 0.50);
    risk.reserve_open_order("sell", "m1", OpenOrderSide::Sell, 0.60, 100.0);
    risk.apply_fill("sell", 100.0, 0.60);
    assert!(risk.daily_pnl() != 0.0);
    risk.reset_daily();
    assert_eq!(risk.daily_pnl(), 0.0);
}

#[test]
fn test_risk_manager_market_position_value() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.reserve_open_order("buy", "m1", OpenOrderSide::Buy, 0.50, 100.0);
    risk.apply_fill("buy", 100.0, 0.50);
    let val = risk.market_position_value("m1");
    assert!((val - 50.0).abs() < 1e-6);
}

#[test]
fn test_risk_manager_market_open_order_value() {
    let mut risk = RiskManager::new(&test_risk_config());
    risk.reserve_open_order("a", "m1", OpenOrderSide::Buy, 0.50, 200.0);
    risk.reserve_open_order("b", "m1", OpenOrderSide::Sell, 0.40, 200.0);
    risk.reserve_open_order("c", "m2", OpenOrderSide::Buy, 0.40, 100.0);
    assert!((risk.market_open_order_value("m1") - 180.0).abs() < 1e-6);
    assert!((risk.total_open_order_value() - 220.0).abs() < 1e-6);
}

#[test]
fn test_risk_manager_apply_fill_returns_none_for_bad_input() {
    let mut risk = RiskManager::new(&test_risk_config());
    assert!(risk.apply_fill("nonexistent", 10.0, 0.50).is_none());
}

// --- SideMode --------------------------------------------------------------

#[test]
fn test_side_mode_two_sided_allows_both() {
    assert!(SideMode::TwoSided.allows_buy());
    assert!(SideMode::TwoSided.allows_sell());
}

#[test]
fn test_side_mode_buy_only() {
    assert!(SideMode::BuyOnly.allows_buy());
    assert!(!SideMode::BuyOnly.allows_sell());
}

#[test]
fn test_side_mode_sell_only() {
    assert!(!SideMode::SellOnly.allows_buy());
    assert!(SideMode::SellOnly.allows_sell());
}

// ---------------------------------------------------------------------------
// Arbitrage concepts: use common math to detect & measure arbitrage
// ---------------------------------------------------------------------------

#[test]
fn test_arbitrage_detection_yes_plus_no_below_one() {
    let yes_price = dec!(0.47);
    let no_price = dec!(0.48);
    let total = yes_price + no_price;
    let settlement = dec!(1);
    let edge = arbitrage_gross_edge_per_share(total, settlement).unwrap();
    assert!(edge > dec!(0));
    assert_eq!(edge, dec!(0.05));
}

#[test]
fn test_arbitrage_detection_no_opportunity_when_total_is_one() {
    let total = dec!(1);
    let settlement = dec!(1);
    let edge = arbitrage_gross_edge_per_share(total, settlement).unwrap();
    assert_eq!(edge, dec!(0));
}

#[test]
fn test_arbitrage_detection_overpriced_basket() {
    let total = dec!(1.12);
    let settlement = dec!(1);
    let edge = arbitrage_gross_edge_per_share(total, settlement).unwrap();
    assert_eq!(edge, dec!(0.12));
}

#[test]
fn test_arbitrage_profit_scales_with_size() {
    let total = dec!(0.94);
    let shares = dec!(1000);
    let gross = arbitrage_gross_edge(total, dec!(1), shares).unwrap();
    // 0.06 edge × 1000 shares = 60
    assert_eq!(gross, dec!(60));
}

#[test]
fn test_arbitrage_kelly_sizing_for_arb_opportunity() {
    let win_prob = dec!(0.55);
    let net_odds = dec!(1);
    let fraction = kelly_fraction(win_prob, net_odds).unwrap();
    let expected = (net_odds * win_prob - (dec!(1) - win_prob)) / net_odds;
    assert!((fraction - expected).abs() < dec!(0.0001));
    assert!(fraction > dec!(0));
}
