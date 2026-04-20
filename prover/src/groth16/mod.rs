//! Groth16 proof system

pub mod prover;
pub mod verifier;

pub use prover::Groth16Prover;
pub use verifier::Groth16Verifier;

/// Groth16 proof
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Proof {
    /// A point
    pub pi_a: [String; 2],
    
    /// B point
    pub pi_b: [[String; 2]; 2],
    
    /// C point
    pub pi_c: [String; 2],
    
    /// Protocol
    pub protocol: String,
}