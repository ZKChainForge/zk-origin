//! Hashing implementations for ZK-ORIGIN

pub mod poseidon;
pub mod merkle;

pub use poseidon::{PoseidonHasher, poseidon_hash, poseidon_hash_two};
pub use merkle::{MerkleTree, MerkleProof};