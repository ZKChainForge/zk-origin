use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use zk_origin::{
    LineageProver,
    OriginClass,
    Transition,
    OriginPolicy,
};

fn bench_single_transition(c: &mut Criterion) {
    let genesis_state = [0u8; 32];

    c.bench_function("single_transition", |b| {
        b.iter(|| {
            let policy = OriginPolicy::default();
            let mut prover = LineageProver::new(policy).unwrap();

            prover.add_transition(Transition::new(
                genesis_state,
                [1u8; 32],
                OriginClass::User,
                1_000,
            ))
            .unwrap();
        });
    });
}

fn bench_multiple_transitions(c: &mut Criterion) {
    let genesis_state = [0u8; 32];
    let counts = [1usize, 5, 10, 50, 100];

    let mut group = c.benchmark_group("transitions");

    for count in counts {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let policy = OriginPolicy::default();
                    let mut prover = LineageProver::new(policy).unwrap();

                    let mut prev_state = genesis_state;

                    for i in 0..count {
                        let new_state = [(i + 1) as u8; 32];

                        prover.add_transition(Transition::new(
                            prev_state,
                            new_state,
                            OriginClass::User,
                            (i + 1) as u64 * 1_000,
                        ))
                        .unwrap();

                        prev_state = new_state;
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_proof_generation(c: &mut Criterion) {
    let genesis_state = [0u8; 32];

    let policy = OriginPolicy::default();
    let mut prover = LineageProver::new(policy).unwrap();

    let mut prev_state = genesis_state;

    for i in 0..10 {
        let new_state = [(i + 1) as u8; 32];

        prover.add_transition(Transition::new(
            prev_state,
            new_state,
            OriginClass::User,
            (i + 1) as u64 * 1_000,
        ))
        .unwrap();

        prev_state = new_state;
    }

    c.bench_function("proof_generation", |b| {
        b.iter(|| {
            prover.finalize().unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_single_transition,
    bench_multiple_transitions,
    bench_proof_generation
);
criterion_main!(benches);