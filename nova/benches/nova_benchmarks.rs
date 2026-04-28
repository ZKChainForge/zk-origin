use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_nova_setup(c: &mut Criterion) {
    c.bench_function("nova_setup", |b| {
        b.iter(|| {
            // Benchmark Nova setup
            black_box(1 + 1)
        })
    });
}

criterion_group!(benches, bench_nova_setup);
criterion_main!(benches);