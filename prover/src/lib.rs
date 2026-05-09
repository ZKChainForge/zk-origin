#![warn(missing_docs)]
//! Complete pipeline:
//! 1. Generate witness from state transitions
//! 2. Create zero-knowledge proofs
//! 3. Verify proofs on-chain
//! 4. Batch operations for efficiency

pub mod error;
pub mod hash;
pub mod witness;

pub use error::{ProverError, Result};
pub use hash::{blake3, hash_multi, sha3_256, Hash, HashType, Hasher};
pub use witness::{TransitionWitness, WitnessGenerator, WitnessSerializer, WitnessValidator};

/// Current version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Module for production constants
pub mod consts {
    /// Number of origin classes
    pub const NUM_ORIGIN_CLASSES: usize = 7;

    /// Genesis origin class constant
    pub const ORIGIN_GENESIS: u8 = 255;

    /// Maximum lineage depth before requiring compression
    pub const MAX_LINEAGE_DEPTH: u32 = 1_000_000;

    /// Typical witness size in bytes
    pub const TYPICAL_WITNESS_SIZE: usize = 8192;

    /// Rate limits per origin class per epoch
    pub const RATE_LIMITS: [u32; 7] = [1, u32::MAX, 10, 100, 5, 1000, 1];
}
