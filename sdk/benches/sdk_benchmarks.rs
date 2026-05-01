// sdk/benches/sdk_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_sdk(c: &mut Criterion) {
    c.bench_function("sdk_init", |b| {
        b.iter(|| {
            let _data = black_box("test");
        });
    });
}

criterion_group!(benches, benchmark_sdk);
criterion_main!(benches);