use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_decimal_ext::DecimalExt;
use rust_decimal_macros::dec;

criterion_group!(benches, benchmark_decimal);
criterion_main!(benches);

fn benchmark_decimal(c: &mut Criterion) {
    c.bench_function("to_unscaled_array_vec", |b| {
        let value = dec!(0.00100000);
        b.iter(|| {
            let result = black_box(&value).to_unscaled_array_vec();
            black_box(result);
        })
    });
    c.bench_function("to_unscaled_array_vec_large", |b| {
        // Mantissa exceeds u64, exercising the 128-bit path.
        let value = rust_decimal::Decimal::MAX;
        b.iter(|| {
            let result = black_box(&value).to_unscaled_array_vec();
            black_box(result);
        })
    });
}
