//! Prover implementations

pub mod lineage_prover;
pub mod witness_gen;
pub mod compress;
pub mod recursive;
pub mod nova_prover;
pub mod commitment_prover;

#[cfg(feature = "real-nova")]
pub mod nova_circuit;

// Re-exports
pub use lineage_prover::{LineageProver, LineageProverBuilder};
pub use witness_gen::WitnessGenerator;

#[cfg(feature = "real-nova")]
pub use nova_prover::{NovaParams, NovaLineageProver, CompressedNovaProof};

#[cfg(feature = "commitment-mode")]
pub use commitment_prover::{CommitmentParams, CommitmentProver};