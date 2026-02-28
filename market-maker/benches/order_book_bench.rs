//! 订单簿性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;

fn bench_orderbook_creation(c: &mut Criterion) {
    c.bench_function("orderbook_creation", |b| {
        b.iter(|| {
            let mut bids = Vec::with_capacity(20);
            let mut asks = Vec::with_capacity(20);
            for i in 0..20 {
                bids.push((
                    Decimal::from_f64_retain(0.50 + i as f64 * 0.01).unwrap(),
                    Decimal::from(100),
                ));
                asks.push((
                    Decimal::from_f64_retain(0.52 + i as f64 * 0.01).unwrap(),
                    Decimal::from(100),
                ));
            }
            black_box((bids, asks));
        })
    });
}

fn bench_spread_calculation(c: &mut Criterion) {
    let bid = Decimal::from_f64_retain(0.50).unwrap();
    let ask = Decimal::from_f64_retain(0.52).unwrap();

    c.bench_function("spread_calculation", |b| {
        b.iter(|| {
            let spread = black_box(ask) - black_box(bid);
            black_box(spread);
        })
    });
}

fn bench_mid_price_calculation(c: &mut Criterion) {
    let bid = Decimal::from_f64_retain(0.50).unwrap();
    let ask = Decimal::from_f64_retain(0.52).unwrap();

    c.bench_function("mid_price_calculation", |b| {
        b.iter(|| {
            let mid = (black_box(bid) + black_box(ask)) / Decimal::from(2);
            black_box(mid);
        })
    });
}

criterion_group!(
    benches,
    bench_orderbook_creation,
    bench_spread_calculation,
    bench_mid_price_calculation
);
criterion_main!(benches);
