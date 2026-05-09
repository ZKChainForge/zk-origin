// prover/benches/prover_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_prover(c: &mut Criterion) {
    c.bench_function("prover_init", |b| {
        b.iter(|| {
            let _data = black_box("test");
        });
    });
}

criterion_group!(benches, benchmark_prover);
criterion_main!(benches);
