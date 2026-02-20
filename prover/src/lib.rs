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
//! # Example
//!
//! ```rust,no_run
//! use zk_origin::{LineageProver, OriginPolicy, Transition, OriginClass};
//!
//! // Create a policy
//! let policy = OriginPolicy::default();
//!
//! // Create prover
//! let mut prover = LineageProver::new(policy).unwrap();
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

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

/// Utility helpers shared across the ZK-ORIGIN library.
///
/// This module contains common helper functions and small abstractions
/// used by the prover, verifier, and circuit layers.
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
pub use verifier::verify::LineageVerifier;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Number of origin classes supported
pub const NUM_ORIGIN_CLASSES: usize = 6;

/// Merkle tree depth for policy tree
pub const POLICY_TREE_DEPTH: usize = 4;

/// Maximum lineage depth supported
pub const MAX_LINEAGE_DEPTH: u64 = 1_000_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_loads() {
        assert!(!VERSION.is_empty());
        assert_eq!(NUM_ORIGIN_CLASSES, 6);
    }
}