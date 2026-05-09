use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use zk_origin_prover::{Hash, WitnessGenerator};

fn bench_witness_generation(c: &mut Criterion) {
    c.bench_function("witness_generation_simple", |b| {
        b.iter(|| {
            let generator = WitnessGenerator::new(Hash::default(), Hash::default());

            generator.generate(
                black_box(Hash::from_array([1u8; 32])),
                black_box(Hash::from_array([2u8; 32])),
                0,
                1,
                Hash::default(),
                Hash::default(),
                vec![0, 0, 0, 0, 0, 0, 0],
                0,
                0,
                1,
                0,
                1000,
                999,
                vec![],
                vec![],
            )
        })
    });
}

fn bench_witness_validation(c: &mut Criterion) {
    c.bench_function("witness_validation", |b| {
        b.iter_batched(
            || {
                let generator = WitnessGenerator::new(Hash::default(), Hash::default());
                generator
                    .generate(
                        Hash::from_array([1u8; 32]),
                        Hash::from_array([2u8; 32]),
                        0,
                        1,
                        Hash::default(),
                        Hash::default(),
                        vec![0, 0, 0, 0, 0, 0, 0],
                        0,
                        0,
                        1,
                        0,
                        1000,
                        999,
                        vec![],
                        vec![],
                    )
                    .unwrap()
            },
            |witness| witness.validate(),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_witness_generation, bench_witness_validation);
criterion_main!(benches);
