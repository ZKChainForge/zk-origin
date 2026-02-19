//! Benchmarks for proof verification

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use zk_origin::{
    LineageProver, LineageVerifier, OriginPolicy, Transition, OriginClass,
};

fn generate_proof(num_steps: usize) -> (zk_origin::LineageProof, [u8; 32], OriginPolicy) {
    let genesis = [0u8; 32];
    let policy = OriginPolicy::default();
    
    let mut prover = LineageProver::new(policy.clone()).unwrap();
    prover.initialize(genesis).unwrap();
    
    for i in 0..num_steps {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(t).unwrap();
    }
    
    (prover.finalize().unwrap(), genesis, policy)
}

fn bench_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify_proof");
    
    for num_steps in [1, 10, 50, 100].iter() {
        let (proof, genesis, policy) = generate_proof(*num_steps);
        let verifier = LineageVerifier::new(genesis, &policy);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(num_steps),
            &(proof, verifier),
            |b, (proof, verifier)| {
                b.iter(|| {
                    verifier.verify(proof).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

fn bench_verify_detailed(c: &mut Criterion) {
    let (proof, genesis, policy) = generate_proof(50);
    let verifier = LineageVerifier::new(genesis, &policy);
    
    c.bench_function("verify_detailed", |b| {
        b.iter(|| {
            verifier.verify_detailed(&proof)
        })
    });
}

fn bench_proof_deserialization(c: &mut Criterion) {
    let (proof, _, _) = generate_proof(50);
    let json = proof.to_json().unwrap();
    
    c.bench_function("proof_from_json", |b| {
        b.iter(|| {
            zk_origin::LineageProof::from_json(&json).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_verify,
    bench_verify_detailed,
    bench_proof_deserialization,
);

criterion_main!(benches);