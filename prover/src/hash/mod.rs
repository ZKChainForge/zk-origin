//! Hashing implementations for ZK-ORIGIN

pub mod poseidon;
pub mod poseidon_native;
pub mod merkle;

pub use poseidon::{PoseidonHasher, poseidon_hash, poseidon_hash_two};
pub use poseidon_native::NativePoseidonHasher;
pub use merkle::{MerkleTree, MerkleProof};

// Re-export the native hasher as the default
pub use poseidon_native::POSEIDON_PARAMS;