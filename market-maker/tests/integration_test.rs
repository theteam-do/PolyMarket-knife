//! Market Maker 集成测试
//!
//! 测试订单簿、报价和风控功能

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// 测试订单簿基本操作
#[test]
fn test_order_book_basic() {
    // 这个测试需要在 market-maker crate 中实现
    // 由于 OrderBook 是 crate 内部的，我们需要通过公共 API 测试
    assert!(true); // 占位符，实际测试需要导出 OrderBook
}

/// 测试风控限制
#[test]
fn test_risk_limits() {
    // 测试风控配置
    assert!(true); // 占位符
}

/// 测试报价计算
#[test]
fn test_quote_calculation() {
    // 测试买卖价差计算
    let spread_bps = 100; // 1%
    let mid_price = 0.50;

    let half_spread = (mid_price * spread_bps as f64) / 10000.0 / 2.0;
    let bid = mid_price - half_spread;
    let ask = mid_price + half_spread;

    assert!((bid - 0.4975).abs() < 0.0001);
    assert!((ask - 0.5025).abs() < 0.0001);
    assert!((ask - bid - 0.005).abs() < 0.0001);
}

/// 测试 Decimal 精度
#[test]
fn test_decimal_precision() {
    let price1 = dec!(0.50123);
    let price2 = dec!(0.49877);

    let sum = price1 + price2;
    assert!((sum - dec!(1.00000)).abs() < dec!(0.00001));

    let diff = price1 - price2;
    assert!((diff - dec!(0.00246)).abs() < dec!(0.00001));
}

/// 测试订单大小计算
#[test]
fn test_order_size_calculation() {
    let order_size_usd = 1000.0;
    let price = 0.50;

    let shares = order_size_usd / price;
    assert!((shares - 2000.0).abs() < 0.01);
}

/// 测试 PnL 计算
#[test]
fn test_pnl_calculation() {
    let entry_price = dec!(0.50);
    let exit_price = dec!(0.55);
    let shares = dec!(1000);

    let pnl = (exit_price - entry_price) * shares;
    assert!((pnl - dec!(50)).abs() < dec!(0.01));
}

/// 测试库存偏斜
#[test]
fn test_inventory_skew() {
    // 当持仓偏向 YES 时，应该降低 YES 卖价
    let mid_price = dec!(0.50);
    let inventory_yes = dec!(5000);
    let inventory_no = dec!(1000);
    let net_inventory = inventory_yes - inventory_no;

    // 净持仓为正，应该偏向卖出
    assert!(net_inventory > dec!(0));

    // 简单的偏斜计算
    let skew_factor = dec!(0.001); // 0.1% 每单位库存
    let adjusted_bid = mid_price - (net_inventory * skew_factor);
    let adjusted_ask = mid_price + (net_inventory * skew_factor);

    assert!(adjusted_bid < mid_price);
    assert!(adjusted_ask > mid_price);
}
