//! Prover implementations for ZK-ORIGIN

pub mod lineage_prover;
pub mod witness_gen;
pub mod recursive;
pub mod compress;

pub use lineage_prover::LineageProver;
pub use witness_gen::WitnessGenerator;