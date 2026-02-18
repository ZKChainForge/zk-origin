//! Main lineage prover implementation

use std::time::Instant;

use crate::types::{
    OriginPolicy, Transition, LineageProof, LineageCommitment,
    CounterCommitment, ProofMetadata,
};
use crate::prover::WitnessGenerator;
use crate::{Result, ZkOriginError};

/// The main prover for generating lineage proofs
/// 
/// This prover accumulates transitions and can generate a proof
/// that verifies the entire lineage is valid.
pub struct LineageProver {
    /// Policy being enforced
    policy: OriginPolicy,
    
    /// Witness generator for creating step witnesses
    witness_gen: WitnessGenerator,
    
    /// Genesis commitment
    genesis_commitment: LineageCommitment,
    
    /// Number of transitions processed
    num_transitions: u64,
    
    /// Whether the prover has been initialized
    initialized: bool,
    
    /// Accumulated proof data (simplified - in production this would be Nova's RecursiveSNARK)
    proof_data: Vec<u8>,
}

impl LineageProver {
    /// Create a new lineage prover with the given policy
    pub fn new(policy: OriginPolicy) -> Result<Self> {
        Ok(Self {
            policy: policy.clone(),
            witness_gen: WitnessGenerator::new(policy),
            genesis_commitment: LineageCommitment::zero(),
            num_transitions: 0,
            initialized: false,
            proof_data: Vec::new(),
        })
    }

    /// Initialize the prover with a genesis state
    pub fn initialize(&mut self, genesis_state_hash: [u8; 32]) -> Result<()> {
        self.witness_gen.reset(genesis_state_hash);
        self.genesis_commitment = LineageCommitment::genesis(genesis_state_hash);
        self.num_transitions = 0;
        self.initialized = true;
        self.proof_data = vec![0u8; 32]; // Placeholder proof data
        
        Ok(())
    }

    /// Add a transition to the lineage
    pub fn add_transition(&mut self, transition: Transition) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Call initialize() before adding transitions".into()
            ));
        }

        // Generate witness (validates transition)
        let witness = self.witness_gen.generate_witness(&transition)?;
        
        // In production, this would:
        // 1. Create a step circuit with the witness
        // 2. Call Nova's prove_step to extend the recursive proof
        // 3. Update the running instance
        
        // For now, we accumulate placeholder proof data
        self.proof_data.extend_from_slice(&witness.compute_transition_hash());
        self.num_transitions += 1;
        
        Ok(())
    }

    /// Add multiple transitions
    pub fn add_transitions(&mut self, transitions: Vec<Transition>) -> Result<()> {
        for transition in transitions {
            self.add_transition(transition)?;
        }
        Ok(())
    }

    /// Check if a transition would be valid without adding it
    pub fn validate_transition(&self, transition: &Transition) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Prover not initialized".into()
            ));
        }
        
        self.witness_gen.would_be_valid(transition)
    }

    /// Finalize and generate the proof
    pub fn finalize(&self) -> Result<LineageProof> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Prover not initialized".into()
            ));
        }

        if self.num_transitions == 0 {
            return Err(ZkOriginError::InvalidLineage(
                "No transitions to prove".into()
            ));
        }

        let start = Instant::now();
        
        // In production, this would:
        // 1. Verify the recursive SNARK
        // 2. Compress to a succinct proof using Spartan
        // 3. Package everything together
        
        // For now, create a placeholder proof
        let proof_bytes = self.create_placeholder_proof();
        
        let proving_time = start.elapsed().as_millis() as u64;
        
        let metadata = ProofMetadata::new()
            .with_proving_time(proving_time);
        
        let proof = LineageProof::new(
            proof_bytes,
            self.witness_gen.current_lineage().clone(),
            self.witness_gen.current_counters().compute_commitment(),
            self.genesis_commitment.clone(),
            self.num_transitions,
            self.policy.compute_hash(),
        ).with_metadata(metadata);
        
        Ok(proof)
    }

    /// Create a placeholder proof (for testing)
    fn create_placeholder_proof(&self) -> Vec<u8> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(b"zk-origin-proof-v1");
        hasher.update(&self.genesis_commitment.value);
        hasher.update(&self.witness_gen.current_lineage().value);
        hasher.update(&self.num_transitions.to_le_bytes());
        hasher.update(&self.proof_data);
        
        hasher.finalize().to_vec()
    }

    /// Get current lineage commitment
    pub fn current_lineage(&self) -> Option<&LineageCommitment> {
        if self.initialized {
            Some(self.witness_gen.current_lineage())
        } else {
            None
        }
    }

    /// Get current depth
    pub fn current_depth(&self) -> u64 {
        self.num_transitions
    }

    /// Get the policy
    pub fn policy(&self) -> &OriginPolicy {
        &self.policy
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Reset the prover
    pub fn reset(&mut self) {
        self.genesis_commitment = LineageCommitment::zero();
        self.num_transitions = 0;
        self.initialized = false;
        self.proof_data.clear();
    }
}

impl Clone for LineageProver {
    fn clone(&self) -> Self {
        // Create a new prover with same policy
        let mut prover = LineageProver::new(self.policy.clone()).unwrap();
        
        if self.initialized {
            prover.genesis_commitment = self.genesis_commitment.clone();
            prover.num_transitions = self.num_transitions;
            prover.initialized = true;
            prover.proof_data = self.proof_data.clone();
            // Note: witness_gen state is not fully cloned, would need enhancement
        }
        
        prover
    }
}

/// Builder for LineageProver
pub struct LineageProverBuilder {
    policy: Option<OriginPolicy>,
    genesis_hash: Option<[u8; 32]>,
}

impl LineageProverBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            policy: None,
            genesis_hash: None,
        }
    }

    /// Set the policy
    pub fn policy(mut self, policy: OriginPolicy) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Set genesis state hash
    pub fn genesis(mut self, hash: [u8; 32]) -> Self {
        self.genesis_hash = Some(hash);
        self
    }

    /// Build the prover
    pub fn build(self) -> Result<LineageProver> {
        let policy = self.policy.unwrap_or_default();
        let mut prover = LineageProver::new(policy)?;
        
        if let Some(genesis) = self.genesis_hash {
            prover.initialize(genesis)?;
        }
        
        Ok(prover)
    }
}

impl Default for LineageProverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OriginClass;

    fn create_prover() -> LineageProver {
        let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();
        prover.initialize([0u8; 32]).unwrap();
        prover
    }

    #[test]
    fn test_prover_creation() {
        let prover = LineageProver::new(OriginPolicy::default());
        assert!(prover.is_ok());
        
        let prover = prover.unwrap();
        assert!(!prover.is_initialized());
    }

    #[test]
    fn test_prover_initialization() {
        let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();
        
        let result = prover.initialize([42u8; 32]);
        assert!(result.is_ok());
        assert!(prover.is_initialized());
    }

    #[test]
    fn test_add_transition() {
        let mut prover = create_prover();
        
        let transition = Transition::new(
            [0u8; 32],
            [1u8; 32],
            OriginClass::User,
            1000,
        );
        
        let result = prover.add_transition(transition);
        assert!(result.is_ok());
        assert_eq!(prover.current_depth(), 1);
    }

    #[test]
    fn test_add_transition_not_initialized() {
        let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();
        
        let transition = Transition::new(
            [0u8; 32],
            [1u8; 32],
            OriginClass::User,
            1000,
        );
        
        let result = prover.add_transition(transition);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZkOriginError::NotInitialized(_)));
    }

    #[test]
    fn test_finalize() {
        let mut prover = create_prover();
        
        // Add some transitions
        for i in 0..5 {
            let transition = Transition::new(
                [i as u8; 32],
                [(i + 1) as u8; 32],
                OriginClass::User,
                1000 + i as u64,
            );
            prover.add_transition(transition).unwrap();
        }
        
        let proof = prover.finalize();
        assert!(proof.is_ok());
        
        let proof = proof.unwrap();
        assert_eq!(proof.num_steps, 5);
        assert!(!proof.proof_bytes.is_empty());
    }

    #[test]
    fn test_finalize_no_transitions() {
        let prover = create_prover();
        
        let result = prover.finalize();
        assert!(result.is_err());
    }

    #[test]
    fn test_policy_violation() {
        let mut prover = create_prover();
        
        // Genesis -> User (valid)
        let t1 = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
        prover.add_transition(t1).unwrap();
        
        // User -> Admin (invalid)
        let t2 = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
        let result = prover.add_transition(t2);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_transition() {
        let mut prover = create_prover();
        
        let valid = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
        assert!(prover.validate_transition(&valid).is_ok());
        
        // Actually add it
        prover.add_transition(valid).unwrap();
        
        // Now User -> Admin should be invalid
        let invalid = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
        assert!(prover.validate_transition(&invalid).is_err());
    }

    #[test]
    fn test_builder() {
        let prover = LineageProverBuilder::new()
            .policy(OriginPolicy::restrictive())
            .genesis([1u8; 32])
            .build();
        
        assert!(prover.is_ok());
        let prover = prover.unwrap();
        assert!(prover.is_initialized());
    }

    #[test]
    fn test_reset() {
        let mut prover = create_prover();
        
        let t = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
        prover.add_transition(t).unwrap();
        
        assert_eq!(prover.current_depth(), 1);
        
        prover.reset();
        
        assert!(!prover.is_initialized());
        assert_eq!(prover.current_depth(), 0);
    }

    #[test]
    fn test_proof_metadata() {
        let mut prover = create_prover();
        
        let t = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
        prover.add_transition(t).unwrap();
        
        let proof = prover.finalize().unwrap();
        
        assert!(proof.metadata.generated_at > 0);
        assert!(!proof.metadata.prover_version.is_empty());
    }
}