//! ZK-ORIGIN: Zero-Knowledge State Lineage Verification
//!
//! This library provides cryptographic verification of state provenance
//! using Nova IVC (Incrementally Verifiable Computation).
//!
//! # Overview
//!
//! ZK-ORIGIN proves that a state has valid lineage - meaning it was derived
//! through a chain of authorized transitions from a known genesis state.
//!
//! # Modes
//!
//! The library supports two modes:
//!
//! 1. **Commitment Mode** (default): Fast hash-based commitments for development
//! 2. **Nova Mode** (with `nova` feature): Full ZK proofs using Nova IVC
//!
//! # Example (Commitment Mode)
//!
//! ```rust,no_run
//! use zk_origin::{LineageProver, OriginPolicy, Transition, OriginClass};
//!
//! // Create a policy
//! let policy = OriginPolicy::default();
//!
//! // Create prover
//! let mut prover = LineageProver::new(policy).unwrap();
//! prover.initialize([0u8; 32]).unwrap();
//!
//! // Add transitions
//! let transition = Transition::new(
//!     [0u8; 32],  // prev_state_hash
//!     [1u8; 32],  // new_state_hash
//!     OriginClass::User,
//!     1000000,    // timestamp
//! );
//! prover.add_transition(transition).unwrap();
//!
//! // Generate proof
//! let proof = prover.finalize().unwrap();
//!
//! // Verify
//! assert!(proof.verify().unwrap());
//! ```
//!
//! # Example (Nova Mode)
//!
//! ```rust,ignore
//! use zk_origin::prover::{NovaParams, NovaLineageProver};
//!
//! // Setup (expensive: ~30 seconds)
//! let params = NovaParams::setup([0u8; 32]).unwrap();
//!
//! // Create prover
//! let mut prover = NovaLineageProver::new(params);
//! prover.initialize([0u8; 32], 0).unwrap();
//!
//! // Add steps (each step: 100-500ms)
//! prover.prove_step(&witness).unwrap();
//!
//! // Finalize (compression: 10-60 seconds)
//! let proof = prover.finalize().unwrap();
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod utils;
pub mod types;
pub mod hash;
pub mod circuit;
pub mod prover;
pub mod verifier;

// Re-export main types for convenience
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

// Nova exports (always available, but expensive to use)
pub use prover::nova_prover::{NovaParams, NovaLineageProver, CompressedNovaProof};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Number of origin classes supported
pub const NUM_ORIGIN_CLASSES: usize = 6;

/// Merkle tree depth for policy tree
pub const POLICY_TREE_DEPTH: usize = 4;

/// Maximum lineage depth supported
pub const MAX_LINEAGE_DEPTH: u64 = 1_000_000;

/// Check if Nova proving is available
pub fn nova_available() -> bool {
    true // Always available in this version
}

/// Get expected proving times
pub fn expected_performance() -> PerformanceEstimates {
    PerformanceEstimates {
        commitment_mode: CommitmentModePerformance {
            add_transition_us: 20,
            finalize_us: 50,
            verify_us: 10,
            proof_size_bytes: 32,
        },
        nova_mode: NovaModePerformance {
            setup_seconds: 15,
            step_ms: 200,
            compression_seconds: 30,
            verify_ms: 20,
            proof_size_bytes: 15000,
        },
    }
}

/// Performance estimates for different modes
#[derive(Debug, Clone)]
pub struct PerformanceEstimates {
    /// Commitment mode performance
    pub commitment_mode: CommitmentModePerformance,
    /// Nova mode performance
    pub nova_mode: NovaModePerformance,
}

/// Commitment mode performance estimates
#[derive(Debug, Clone)]
pub struct CommitmentModePerformance {
    /// Time to add a transition (microseconds)
    pub add_transition_us: u64,
    /// Time to finalize (microseconds)
    pub finalize_us: u64,
    /// Time to verify (microseconds)
    pub verify_us: u64,
    /// Proof size in bytes
    pub proof_size_bytes: usize,
}

/// Nova mode performance estimates
#[derive(Debug, Clone)]
pub struct NovaModePerformance {
    /// Setup time (seconds)
    pub setup_seconds: u64,
    /// Time per step (milliseconds)
    pub step_ms: u64,
    /// Compression time (seconds)
    pub compression_seconds: u64,
    /// Verification time (milliseconds)
    pub verify_ms: u64,
    /// Proof size in bytes
    pub proof_size_bytes: usize,
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
    fn test_nova_available() {
        assert!(nova_available());
    }

    #[test]
    fn test_performance_estimates() {
        let perf = expected_performance();
        assert!(perf.nova_mode.step_ms > perf.commitment_mode.add_transition_us);
    }
}