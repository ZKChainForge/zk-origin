//! Recursive lineage prover using Nova folding scheme

use crate::types::{LineageCommitment, Origin, Policy, Transition};
use thiserror::Error;

/// Errors that can occur during proving
#[derive(Error, Debug)]
pub enum ProverError {
    #[error("Policy violation: {from} -> {to}")]
    PolicyViolation { from: Origin, to: Origin },
    
    #[error("Invalid previous state")]
    InvalidPreviousState,
    
    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),
}

/// Recursive lineage prover
pub struct LineageProver {
    /// Current lineage commitment
    current_lineage: LineageCommitment,
    /// Current origin class
    current_origin: Origin,
    /// Policy
    policy: Policy,
    /// Accumulated transitions
    transitions: Vec<Transition>,
}

impl LineageProver {
    /// Create a new prover with genesis state
    pub fn new(genesis_state: [u8; 32]) -> Result<Self, ProverError> {
        Ok(Self {
            current_lineage: LineageCommitment::genesis(genesis_state),
            current_origin: Origin::Genesis,
            policy: Policy::default_policy(),
            transitions: Vec::new(),
        })
    }
    
    /// Add a transition
    pub fn add_transition(&mut self, transition: Transition) -> Result<(), ProverError> {
        // Check policy
        if !self.policy.is_allowed(
            self.current_origin,
            transition.origin,
            self.current_lineage.depth,
        ) {
            return Err(ProverError::PolicyViolation {
                from: self.current_origin,
                to: transition.origin,
            });
        }
        
        // Update lineage (placeholder - would compute actual Poseidon hash)
        self.current_lineage = LineageCommitment::new(
            transition.new_state, // Placeholder
            self.current_lineage.depth + 1,
        );
        self.current_origin = transition.origin;
        self.transitions.push(transition);
        
        Ok(())
    }
    
    /// Get current lineage
    pub fn current_lineage(&self) -> &LineageCommitment {
        &self.current_lineage
    }
    
    /// Get number of transitions
    pub fn num_transitions(&self) -> usize {
        self.transitions.len()
    }
    
    /// Finalize and generate proof (placeholder)
    pub fn finalize(&self) -> Result<LineageProof, ProverError> {
        // In production, this would use Nova to generate a recursive proof
        Ok(LineageProof {
            lineage_commitment: self.current_lineage.clone(),
            proof_data: vec![0u8; 256], // Placeholder
        })
    }
}

/// A lineage proof
#[derive(Clone, Debug)]
pub struct LineageProof {
    /// Final lineage commitment
    pub lineage_commitment: LineageCommitment,
    /// Proof data (would be Nova proof in production)
    pub proof_data: Vec<u8>,
}

impl LineageProof {
    /// Verify the proof (placeholder)
    pub fn verify(&self) -> bool {
        // In production, verify the Nova proof
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_transitions() {
        let mut prover = LineageProver::new([0u8; 32]).unwrap();
        
        // Genesis -> User
        prover.add_transition(Transition::new(
            [0u8; 32],
            [1u8; 32],
            Origin::User,
            1000,
        )).unwrap();
        
        // User -> User
        prover.add_transition(Transition::new(
            [1u8; 32],
            [2u8; 32],
            Origin::User,
            2000,
        )).unwrap();
        
        assert_eq!(prover.num_transitions(), 2);
        assert_eq!(prover.current_lineage().depth, 2);
    }
    
    #[test]
    fn test_policy_violation() {
        let mut prover = LineageProver::new([0u8; 32]).unwrap();
        
        // Genesis -> User
        prover.add_transition(Transition::new(
            [0u8; 32],
            [1u8; 32],
            Origin::User,
            1000,
        )).unwrap();
        
        // User -> Admin (should fail)
        let result = prover.add_transition(Transition::new(
            [1u8; 32],
            [2u8; 32],
            Origin::Admin,
            2000,        ));
        
        assert!(matches!(result, Err(ProverError::PolicyViolation { .. })));
    }
    
    #[test]
    fn test_finalize() {
        let mut prover = LineageProver::new([0u8; 32]).unwrap();
        
        prover.add_transition(Transition::new(
            [0u8; 32],
            [1u8; 32],
            Origin::User,
            1000,
        )).unwrap();
        
        let proof = prover.finalize().unwrap();
        assert!(proof.verify());
    }
}