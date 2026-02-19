//! Benchmarks for proof generation

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use zk_origin::{LineageProver, OriginPolicy, Transition, OriginClass};

fn bench_add_transition(c: &mut Criterion) {
    let policy = OriginPolicy::default();
    
    c.bench_function("add_transition", |b| {
        b.iter_batched(
            || {
                let mut prover = LineageProver::new(policy.clone()).unwrap();
                prover.initialize([0u8; 32]).unwrap();
                prover
            },
            |mut prover| {
                let t = Transition::new(
                    [0u8; 32],
                    [1u8; 32],
                    OriginClass::User,
                    1000,
                );
                prover.add_transition(t).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_prove_n_steps(c: &mut Criterion) {
    let policy = OriginPolicy::default();
    
    let mut group = c.benchmark_group("prove_n_steps");
    
    for num_steps in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_steps),
            num_steps,
            |b, &num_steps| {
                b.iter_batched(
                    || {
                        let mut prover = LineageProver::new(policy.clone()).unwrap();
                        prover.initialize([0u8; 32]).unwrap();
                        
                        for i in 0..num_steps {
                            let t = Transition::new(
                                [i as u8; 32],
                                [(i + 1) as u8; 32],
                                OriginClass::User,
                                (i as u64 + 1) * 1000,
                            );
                            prover.add_transition(t).unwrap();
                        }
                        
                        prover
                    },
                    |prover| {
                        prover.finalize().unwrap()
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

fn bench_witness_generation(c: &mut Criterion) {
    use zk_origin::prover::WitnessGenerator;
    
    let policy = OriginPolicy::default();
    
    c.bench_function("witness_generation", |b| {
        b.iter_batched(
            || {
                let mut gen = WitnessGenerator::new(policy.clone());
                gen.reset([0u8; 32]);
                gen
            },
            |mut gen| {
                let t = Transition::new(
                    [0u8; 32],
                    [1u8; 32],
                    OriginClass::User,
                    1000,
                );
                gen.generate_witness(&t).unwrap()
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_add_transition,
    bench_prove_n_steps,
    bench_witness_generation,
);

criterion_main!(benches);