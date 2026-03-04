//! Market Maker 性能基准测试
//!
//! 使用 criterion 进行性能基准测试

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// 基准测试：订单簿中间价计算
fn bench_orderbook_mid_price(c: &mut Criterion) {
    c.bench_function("orderbook_mid_price", |b| {
        b.iter(|| {
            let best_bid = 0.49;
            let best_ask = 0.51;
            let _mid = (best_bid + best_ask) / 2.0;
        })
    });
}

/// 基准测试：Decimal vs f64 性能对比
fn bench_decimal_vs_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal_vs_f64");

    group.bench_function("f64_addition", |b| {
        b.iter(|| {
            let _result = black_box(0.50) + black_box(0.50);
        })
    });

    group.bench_function("decimal_addition", |b| {
        b.iter(|| {
            let a = dec!(0.50);
            let b = dec!(0.50);
            let _result = a + b;
        })
    });

    group.bench_function("f64_multiplication", |b| {
        b.iter(|| {
            let _result = black_box(1000.0) * black_box(0.50);
        })
    });

    group.bench_function("decimal_multiplication", |b| {
        b.iter(|| {
            let a = dec!(1000.0);
            let b = dec!(0.50);
            let _result = a * b;
        })
    });

    group.finish();
}

/// 基准测试：报价计算
fn bench_quote_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("quote_calculation");

    for spread_bps in [50, 100, 200] {
        group.bench_with_input(
            BenchmarkId::new("spread_bps", spread_bps),
            &spread_bps,
            |b, spread_bps| {
                b.iter(|| {
                    let mid_price = 0.50;
                    let half_spread = (mid_price * *spread_bps as f64) / 10000.0 / 2.0;
                    let _bid = mid_price - half_spread;
                    let _ask = mid_price + half_spread;
                })
            },
        );
    }

    group.finish();
}

/// 基准测试：PnL 计算
fn bench_pnl_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pnl_calculation");

    group.bench_function("simple_pnl", |b| {
        b.iter(|| {
            let entry = dec!(0.50);
            let exit = dec!(0.55);
            let shares = dec!(1000);
            let _pnl = (exit - entry) * shares;
        })
    });

    group.bench_function("complex_pnl", |b| {
        b.iter(|| {
            // 模拟多次交易的 PnL 计算
            let mut total_pnl = dec!(0);
            for i in 0..10 {
                let i_dec = Decimal::from(i);
                let entry = dec!(0.50) + dec!(0.01) * i_dec;
                let exit = dec!(0.55) + dec!(0.01) * i_dec;
                let shares = dec!(100);
                total_pnl += (exit - entry) * shares;
            }
            black_box(total_pnl);
        })
    });

    group.finish();
}

/// 基准测试：风控检查
fn bench_risk_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("risk_checks");

    group.bench_function("position_limit_check", |b| {
        b.iter(|| {
            let current_position = dec!(5000);
            let max_position = dec!(10000);
            let _can_trade = current_position < max_position;
        })
    });

    group.bench_function("loss_limit_check", |b| {
        b.iter(|| {
            let daily_pnl = dec!(-200);
            let max_loss = dec!(500);
            let _can_trade = daily_pnl > -max_loss;
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_orderbook_mid_price,
    bench_decimal_vs_f64,
    bench_quote_calculation,
    bench_pnl_calculation,
    bench_risk_checks,
);

criterion_main!(benches);
