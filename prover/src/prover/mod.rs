//! Prover implementations
//!
//! This module provides two proving backends:
//! - `commitment_prover`: Fast hash-based commitments (NOT ZK)
//! - `nova_prover`: Real Nova IVC proofs (slow but real ZK)

pub mod lineage_prover;
pub mod witness_gen;
pub mod compress;
pub mod recursive;

// Always compile both modules, but some functionality is feature-gated
pub mod nova_prover;
pub mod commitment_prover;
pub use nova_prover::NovaLineageProver;

// Re-exports
pub use lineage_prover::{LineageProver, LineageProverBuilder};
pub use witness_gen::WitnessGenerator;

// Feature-gated re-exports
#[cfg(feature = "real-nova")]
pub use nova_prover::{NovaParams, NovaLineageProver, CompressedNovaProof};

#[cfg(feature = "commitment-mode")]
pub use commitment_prover::{CommitmentParams, CommitmentProver};