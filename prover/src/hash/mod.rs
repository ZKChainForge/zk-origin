//! Hashing implementations for ZK-ORIGIN

pub mod merkle;
pub mod poseidon;
pub mod poseidon_native;

pub use merkle::{MerkleProof, MerkleTree};
pub use poseidon::{poseidon_hash, poseidon_hash_two, PoseidonHasher};
pub use poseidon_native::NativePoseidonHasher;

// Re-export the native hasher as the default
pub use poseidon_native::POSEIDON_PARAMS;
