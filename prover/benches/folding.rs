//! Benchmarks for folding operations (placeholder)

use criterion::{criterion_group, criterion_main, Criterion};
use zk_origin::hash::{PoseidonHasher, MerkleTree};

fn bench_poseidon_hash(c: &mut Criterion) {
    let hasher = PoseidonHasher::new();
    let input1 = [1u8; 32];
    let input2 = [2u8; 32];
    
    c.bench_function("poseidon_hash_two", |b| {
        b.iter(|| {
            hasher.hash_two(&input1, &input2)
        })
    });
}

fn bench_poseidon_hash_five(c: &mut Criterion) {
    let hasher = PoseidonHasher::new();
    let inputs = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32], [5u8; 32]];
    
    c.bench_function("poseidon_hash_five", |b| {
        b.iter(|| {
            hasher.hash(&inputs)
        })
    });
}

fn bench_merkle_tree_build(c: &mut Criterion) {
    let leaves: Vec<[u8; 32]> = (0..16)
        .map(|i| {
            let mut arr = [0u8; 32];
            arr[0] = i;
            arr
        })
        .collect();
    
    c.bench_function("merkle_build_16", |b| {
        b.iter(|| {
            MerkleTree::new(leaves.clone())
        })
    });
}

fn bench_merkle_prove(c: &mut Criterion) {
    let leaves: Vec<[u8; 32]> = (0..16)
        .map(|i| {
            let mut arr = [0u8; 32];
            arr[0] = i;
            arr
        })
        .collect();
    
    let tree = MerkleTree::new(leaves);
    
    c.bench_function("merkle_prove", |b| {
        b.iter(|| {
            tree.prove(7).unwrap()
        })
    });
}

fn bench_merkle_verify(c: &mut Criterion) {
    let leaves: Vec<[u8; 32]> = (0..16)
        .map(|i| {
            let mut arr = [0u8; 32];
            arr[0] = i;
            arr
        })
        .collect();
    
    let tree = MerkleTree::new(leaves);
    let proof = tree.prove(7).unwrap();
    
    c.bench_function("merkle_verify", |b| {
        b.iter(|| {
            proof.verify()
        })
    });
}

criterion_group!(
    benches,
    bench_poseidon_hash,
    bench_poseidon_hash_five,
    bench_merkle_tree_build,
    bench_merkle_prove,
    bench_merkle_verify,
);

criterion_main!(benches);