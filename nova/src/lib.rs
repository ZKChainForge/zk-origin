#![warn(missing_docs)]
#![cfg_attr(test, allow(dead_code))]
//! Provides constant-size proofs for arbitrary lineage depth through
//! recursive composition and Nova folding.
//!
//! # Overview
//!
//! This library implements a Nova IVC (Iterative Interactive Proofs) system that allows
//! generating constant-size proofs for computational lineages of arbitrary depth.
//!
//! # Example
//!
//! ```ignore
//! use zk_origin_nova::{NovaConfig, NovaIVCProver, NovaVerifier};
//!
//! // Create a prover with testing configuration
//! let config = NovaConfig::testing();
//! let mut prover = NovaIVCProver::new(config)?;
//!
//! // Add transitions
//! let state = vec![0u8; 48];
//! prover.add_transition(&state)?;
//!
//! // Finalize and get proof
//! let proof = prover.finalize()?;
//!
//! // Verify the proof
//! let valid = NovaVerifier::verify(&proof, &proof.genesis_state, &proof.final_state)?;
//! assert!(valid);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

/// Compression module for Nova to Groth16 conversion
pub mod compression;

/// Configuration module for Nova IVC
pub mod config;

/// Error handling module
pub mod error;

/// Cryptographic hashing module
pub mod hash;

/// Nova IVC prover implementation
pub mod nova_ivc;

/// Proof verification module
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_consts() {
        assert_eq!(consts::STATE_SIZE, 48);
        assert_eq!(consts::LINEAGE_COMMITMENT_INDEX, 0);
        assert_eq!(consts::COUNTER_COMMITMENT_INDEX, 32);
    }
}