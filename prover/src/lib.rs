//! ZK-Origin Prover Library
//!
//! Provides zero-knowledge proofs for data lineage verification.
//!
//! ## Features
//! - `real-nova`: Use Nova IVC for real ZK proofs (slow, ~30-120s setup)
//! - `commitment-mode`: Use hash commitments (fast, but NOT zero-knowledge)
//!
//! Enable exactly ONE of these features.

#![warn(missing_docs)]

#[cfg(all(feature = "real-nova", feature = "commitment-mode"))]
compile_error!("Enable only one of 'real-nova' or 'commitment-mode'");

#[cfg(not(any(feature = "real-nova", feature = "commitment-mode")))]
compile_error!("Enable either 'real-nova' or 'commitment-mode' feature");

pub mod hash;
pub mod prover;
pub mod types;
pub mod verifier;

// Re-export error types from types module (not a separate top-level module)
pub use types::error::{Result, ZkOriginError};

// Re-export main types
pub use types::{LineageCommitment, LineageProof, OriginClass, OriginPolicy, Transition};

// Re-export prover types
pub use prover::{LineageProver, LineageProverBuilder, WitnessGenerator};

#[cfg(feature = "real-nova")]
pub use prover::{CompressedNovaProof, NovaLineageProver, NovaParams};

#[cfg(feature = "commitment-mode")]
pub use prover::{CommitmentParams, CommitmentProver};



// Re-export verifier
pub use verifier::LineageVerifier;

/// Get the current proving mode as a string
pub fn proving_mode() -> &'static str {
    #[cfg(feature = "real-nova")]
    return "Nova IVC (Real ZK)";

    #[cfg(feature = "commitment-mode")]
    return "Hash Commitments (NOT ZK)";
}

/// Check if real ZK is enabled
pub fn is_real_zk_enabled() -> bool {
    cfg!(feature = "real-nova")
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proving_mode() {
        let mode = proving_mode();
        assert!(!mode.is_empty());
        println!("Proving mode: {}", mode);
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        println!("Library version: {}", VERSION);
    }

    #[test]
    #[cfg(feature = "commitment-mode")]
    fn test_commitment_mode_enabled() {
        assert!(!is_real_zk_enabled());
        assert!(proving_mode().contains("NOT ZK"));
    }

    #[test]
    #[cfg(feature = "real-nova")]
    fn test_nova_mode_enabled() {
        assert!(is_real_zk_enabled());
        assert!(proving_mode().contains("Nova"));
    }

    #[test]
    fn test_basic_types() {
        // Create a default policy (it already has rules)
        let _policy = OriginPolicy::default();

        // Test transition
        let transition = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
        assert_eq!(transition.timestamp, 1000);
        assert_eq!(transition.origin_class, OriginClass::User);

        // Test commitment
        let commitment = LineageCommitment::genesis([42u8; 32]);
        assert!(commitment.is_genesis());
    }

    #[test]
    fn test_error_types() {
        let err = ZkOriginError::InvalidLineage("test".into());
        assert!(err.to_string().contains("Invalid lineage"));
    }
}
