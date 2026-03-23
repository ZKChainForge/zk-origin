//! ZK-Origin Prover Library
//!
//! Provides zero-knowledge proofs for data lineage verification.
//!
//! ## Features
//! - `real-nova`: Use Nova IVC for real ZK proofs (~10KB, incremental)
//! - `commitment-mode`: Use hash commitments (fast, but NOT zero-knowledge)
//! - `compact-zk`: Use Groth16 for compact ZK proofs (<1KB)
//!
//! Enable exactly ONE of these features.

#![warn(missing_docs)]

#[cfg(all(feature = "real-nova", feature = "commitment-mode"))]
compile_error!("Enable only one of 'real-nova', 'commitment-mode', or 'compact-zk'");

#[cfg(all(feature = "real-nova", feature = "compact-zk"))]
compile_error!("Enable only one of 'real-nova', 'commitment-mode', or 'compact-zk'");

#[cfg(all(feature = "commitment-mode", feature = "compact-zk"))]
compile_error!("Enable only one of 'real-nova', 'commitment-mode', or 'compact-zk'");

#[cfg(not(any(feature = "real-nova", feature = "commitment-mode", feature = "compact-zk")))]
compile_error!("Enable one of 'real-nova', 'commitment-mode', or 'compact-zk' feature");

pub mod hash;
pub mod prover;
pub mod types;
pub mod verifier;

// Re-export error types from types module
pub use types::error::{Result, ZkOriginError};

// Re-export main types
pub use types::{LineageCommitment, LineageProof, OriginClass, OriginPolicy, Transition, StepWitness};

// Re-export prover types
pub use prover::{LineageProver, LineageProverBuilder, WitnessGenerator};

#[cfg(feature = "real-nova")]
pub use prover::{CompressedNovaProof, NovaLineageProver, NovaParams};

#[cfg(feature = "commitment-mode")]
pub use prover::{CommitmentParams, CommitmentProver};

#[cfg(feature = "compact-zk")]
pub use prover::{
    CompactLineageCircuit, Groth16LineageProver, Groth16Params, TransitionWitness,
    verify_groth16_proof, MAX_TRANSITIONS,
};

// Re-export verifier
pub use verifier::LineageVerifier;

/// Get the current proving mode as a string
pub fn proving_mode() -> &'static str {
    #[cfg(feature = "compact-zk")]
    return "Groth16 (Compact ZK <1KB)";

    #[cfg(feature = "real-nova")]
    return "Nova IVC (Real ZK)";

    #[cfg(feature = "commitment-mode")]
    return "Hash Commitments (NOT ZK)";
}

/// Check if real ZK is enabled (Nova or Groth16)
pub fn is_real_zk_enabled() -> bool {
    cfg!(feature = "real-nova") || cfg!(feature = "compact-zk")
}

/// Check if compact ZK (Groth16) is enabled
pub fn is_compact_zk_enabled() -> bool {
    cfg!(feature = "compact-zk")
}

/// Check if Nova is enabled
pub fn is_nova_enabled() -> bool {
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
    #[cfg(feature = "compact-zk")]
    fn test_compact_zk_enabled() {
        assert!(is_real_zk_enabled());
        assert!(is_compact_zk_enabled());
        assert!(proving_mode().contains("Groth16"));
    }

    #[test]
    fn test_basic_types() {
        let _policy = OriginPolicy::default();

        let transition = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
        assert_eq!(transition.timestamp, 1000);
        assert_eq!(transition.origin_class, OriginClass::User);

        let commitment = LineageCommitment::genesis([42u8; 32]);
        assert!(commitment.is_genesis());
    }

    #[test]
    fn test_error_types() {
        let err = ZkOriginError::InvalidLineage("test".into());
        assert!(err.to_string().contains("Invalid lineage"));
    }
}