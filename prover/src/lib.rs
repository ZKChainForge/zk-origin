//! # ZK-ORIGIN: Zero-Knowledge State Lineage Verification
//!
//! ## Implementation Modes
//!
//! ### Commitment Mode (default)
//! - Uses hash-based commitments
//! - **NOT cryptographically zero-knowledge**
//! - Fast: microseconds per operation
//!
//! ### Real Nova Mode (`real-nova` feature)
//! - Uses Nova IVC for actual ZK proofs
//! - **Cryptographically secure**
//! - Slow: seconds per operation
//!
//! ## Building
//!
//! ```bash
//! # Development (fast, not real ZK)
//! cargo build
//!
//! # Production (real ZK proofs)
//! cargo build --features real-nova --no-default-features
//! ```

#![warn(missing_docs)]

pub mod types;
pub mod hash;
pub mod circuit;
pub mod prover;
pub mod verifier;

// Re-export main types
pub use types::{
    origin::OriginClass,
    lineage::LineageCommitment,
    transition::Transition,
    policy::OriginPolicy,
    witness::StepWitness,
    proof::LineageProof,
    error::{ZkOriginError, Result},
};

pub use prover::lineage_prover::LineageProver;
pub use prover::witness_gen::WitnessGenerator;
pub use verifier::verify::LineageVerifier;

// Conditional exports based on features
#[cfg(feature = "real-nova")]
pub use prover::nova_prover::{NovaParams, NovaLineageProver, CompressedNovaProof};

#[cfg(feature = "commitment-mode")]
pub use prover::commitment_prover::{CommitmentParams, CommitmentProver};

// Always export these for checking mode at runtime
pub use prover::nova_prover as nova;
pub use prover::commitment_prover as commitment;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Number of origin classes supported
pub const NUM_ORIGIN_CLASSES: usize = 6;

/// Merkle tree depth for policy tree
pub const POLICY_TREE_DEPTH: usize = 4;

/// Maximum lineage depth supported
pub const MAX_LINEAGE_DEPTH: u64 = 1_000_000;

/// Check if real Nova ZK proving is enabled
pub fn is_real_zk_enabled() -> bool {
    cfg!(feature = "real-nova")
}

/// Check if commitment mode is enabled
pub fn is_commitment_mode() -> bool {
    cfg!(feature = "commitment-mode") || !cfg!(feature = "real-nova")
}

/// Get the current proving mode as a string
pub fn proving_mode() -> &'static str {
    if cfg!(feature = "real-nova") {
        "Nova IVC (Real ZK)"
    } else {
        "Commitment Mode (Not ZK)"
    }
}

/// Expected performance for current mode
pub fn expected_performance() -> PerformanceEstimates {
    if cfg!(feature = "real-nova") {
        PerformanceEstimates {
            setup_time: "30-120 seconds".to_string(),
            step_time: "500-2000 ms".to_string(),
            compression_time: "10-60 seconds".to_string(),
            verification_time: "10-50 ms".to_string(),
            proof_size: "10-20 KB".to_string(),
            is_real_zk: true,
        }
    } else {
        PerformanceEstimates {
            setup_time: "< 1 ms".to_string(),
            step_time: "10-50 µs".to_string(),
            compression_time: "< 1 ms".to_string(),
            verification_time: "< 1 µs".to_string(),
            proof_size: "32 bytes".to_string(),
            is_real_zk: false,
        }
    }
}

/// Performance estimates
#[derive(Debug, Clone)]
pub struct PerformanceEstimates {
    /// Expected setup time
    pub setup_time: String,
    /// Expected time per step
    pub step_time: String,
    /// Expected compression time
    pub compression_time: String,
    /// Expected verification time
    pub verification_time: String,
    /// Expected proof size
    pub proof_size: String,
    /// Whether this is real ZK
    pub is_real_zk: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_loads() {
        assert!(!VERSION.is_empty());
        assert_eq!(NUM_ORIGIN_CLASSES, 6);
    }

    #[test]
    fn test_mode_detection() {
        let mode = proving_mode();
        assert!(!mode.is_empty());
        println!("Current proving mode: {}", mode);
    }
}