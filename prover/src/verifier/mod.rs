//! Verification logic for ZK-ORIGIN proofs

pub mod public_inputs;
pub mod verify;

pub use public_inputs::PublicInputs;
pub use verify::{
    verify_proof, 
    verify_proof_self_consistent, 
    LineageVerifier, 
    VerificationResult
};

#[cfg(feature = "real-nova")]
pub use verify::{verify_zk_proof, verify_zk_proof_self_consistent};