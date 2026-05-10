#![warn(missing_docs)]
//! Provides constant-size proofs for arbitrary lineage depth through
//! recursive composition and Nova folding.

pub mod compression;
pub mod config;
pub mod error;
pub mod hash;
pub mod nova_ivc;
pub mod verification;

pub use compression::{CompressionStats, Groth16Proof, NovaCompressor};
pub use config::NovaConfig;
pub use error::{NovaError, Result};
pub use hash::{blake3, sha3_256, verify_hash, Hash, HashType, Hasher};
pub use nova_ivc::{CompressedNovaProof, NovaIVCProver, STATE_SIZE};
pub use verification::{NovaVerifier, ProofStats};

/// Current version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Module for production constants
pub mod consts {
    /// State vector size (6 elements * 8 bytes)
    pub const STATE_SIZE: usize = 48;

    /// Lineage commitment index in state
    pub const LINEAGE_COMMITMENT_INDEX: usize = 0;

    /// Counter commitment index in state
    pub const COUNTER_COMMITMENT_INDEX: usize = 32;

    /// Typical compressed proof size
    pub const TYPICAL_PROOF_SIZE: usize = 2500;

    /// Maximum steps in a single proof
    pub const MAX_STEPS: usize = 1_000_000;
}
