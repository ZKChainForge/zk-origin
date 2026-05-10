use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use zk_origin_nova::{NovaConfig, NovaIVCProver, NovaVerifier};

fn bench_prover_creation(c: &mut Criterion) {
    c.bench_function("nova_prover_creation", |b| {
        b.iter(|| {
            let config = NovaConfig::testing();
            let _ = NovaIVCProver::new(black_box(config));
        })
    });
}

fn bench_add_transition(c: &mut Criterion) {
    let mut group = c.benchmark_group("nova_add_transition");

    for step_count in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &step_count| {
                b.iter_batched(
                    || {
                        let config = NovaConfig::testing();
                        NovaIVCProver::new(config).unwrap()
                    },
                    |mut prover| {
                        for i in 0..step_count {
                            let mut state = vec![0u8; 48];
                            state[0] = (i % 256) as u8;
                            let _ = prover.add_transition(black_box(&state));
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

fn bench_finalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("nova_finalize");

    for step_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &step_count| {
                b.iter_batched(
                    || {
                        let mut config = NovaConfig::testing();
                        config.max_steps = 10000;
                        let mut prover = NovaIVCProver::new(config).unwrap();

                        for i in 0..step_count {
                            let mut state = vec![0u8; 48];
                            state[0] = (i % 256) as u8;
                            let _ = prover.add_transition(&state);
                        }

                        prover
                    },
                    |prover| {
                        let _ = prover.finalize();
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

fn bench_verify(c: &mut Criterion) {
    c.bench_function("nova_verify", |b| {
        b.iter_batched(
            || {
                let config = NovaConfig::testing();
                let mut prover = NovaIVCProver::new(config).unwrap();

                for i in 0..10 {
                    let mut state = vec![0u8; 48];
                    state[0] = (i % 256) as u8;
                    let _ = prover.add_transition(&state);
                }

                prover.finalize().unwrap()
            },
            |proof| {
                let _ = NovaVerifier::verify(
                    black_box(&proof),
                    black_box(&proof.genesis_state),
                    black_box(&proof.final_state),
                );
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_prover_creation,
    bench_add_transition,
    bench_finalize,
    bench_verify
);
criterion_main!(benches);
