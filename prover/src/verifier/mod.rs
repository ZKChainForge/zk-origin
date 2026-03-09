//! Verification logic for ZK-ORIGIN proofs

pub mod public_inputs;
pub mod verify;

pub use public_inputs::PublicInputs;
pub use verify::LineageVerifier;
