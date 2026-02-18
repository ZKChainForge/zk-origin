//! Verification logic for ZK-ORIGIN proofs

pub mod verify;
pub mod public_inputs;

pub use verify::LineageVerifier;
pub use public_inputs::PublicInputs;