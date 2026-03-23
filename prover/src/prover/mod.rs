//! Prover implementations

pub mod commitment_prover;
pub mod compress;
pub mod lineage_prover;
/// Nova prover implementation and proof structures.
pub mod nova_prover;
pub mod recursive;
pub mod witness_gen;

#[cfg(feature = "real-nova")]
pub mod nova_circuit;

#[cfg(feature = "compact-zk")]
pub mod groth16_circuit;

pub mod groth16_prover;

// Re-exports
pub use lineage_prover::{LineageProver, LineageProverBuilder};
pub use witness_gen::WitnessGenerator;

#[cfg(feature = "real-nova")]
pub use nova_prover::{CompressedNovaProof, NovaLineageProver, NovaParams};

#[cfg(feature = "commitment-mode")]
pub use commitment_prover::{CommitmentParams, CommitmentProver};

#[cfg(feature = "compact-zk")]
pub use groth16_circuit::{CompactLineageCircuit, TransitionWitness, MAX_TRANSITIONS};

pub use groth16_prover::{Groth16LineageProver, Groth16Params};

#[cfg(feature = "compact-zk")]
pub use groth16_prover::verify_groth16_proof;