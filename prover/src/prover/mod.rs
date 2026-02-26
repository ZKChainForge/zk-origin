//! Prover implementations for ZK-ORIGIN

pub mod lineage_prover;
pub mod witness_gen;
pub mod recursive;
pub mod compress;
pub mod nova_prover;

pub use lineage_prover::LineageProver;
pub use witness_gen::WitnessGenerator;
pub use nova_prover::{NovaLineageProver, NovaParams, CompressedNovaProof};