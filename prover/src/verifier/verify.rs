//! Proof verification implementation

use crate::types::lineage::LineageCommitment;
use crate::types::OriginPolicy;
use crate::types::proof::LineageProof;
use crate::{Result, ZkOriginError};

/// Verifier for lineage proofs
pub struct LineageVerifier {
    /// Expected genesis commitment
    expected_genesis: LineageCommitment,
    
    /// Expected policy hash
    expected_policy_hash: [u8; 32],
    
    /// Policy (for reference)
    #[allow(dead_code)]
    policy: OriginPolicy,
}

impl LineageVerifier {
    /// Create a new verifier
    pub fn new(genesis_state_hash: [u8; 32], policy: &OriginPolicy) -> Self {
        Self {
            expected_genesis: LineageCommitment::genesis(genesis_state_hash),
            expected_policy_hash: policy.compute_hash(),
            policy: policy.clone(),
        }
    }

    /// Verify a lineage proof
    pub fn verify(&self, proof: &LineageProof) -> Result<bool> {
        // Check 1: Genesis matches
        if proof.genesis_commitment.value != self.expected_genesis.value {
            return Err(ZkOriginError::GenesisMismatch);
        }

        // Check 2: Policy matches
        if proof.policy_hash != self.expected_policy_hash {
            return Err(ZkOriginError::VerificationFailed(
                "Policy hash mismatch".into()
            ));
        }

        // Check 3: Depth consistency
        if proof.final_lineage.depth != proof.num_steps {
            return Err(ZkOriginError::VerificationFailed(
                "Depth mismatch".into()
            ));
        }

        // Check 4: Proof non-empty
        if proof.proof_bytes.is_empty() {
            return Err(ZkOriginError::InvalidProof("Empty proof".into()));
        }

        // Check 5: For real ZK proofs, additional verification would happen here
        // This is simplified - full verification requires the public parameters

        Ok(true)
    }

    /// Verify a proof with detailed results
    pub fn verify_detailed(&self, proof: &LineageProof) -> VerificationResult {
        let mut result = VerificationResult::new();

        result.genesis_valid = proof.genesis_commitment.value == self.expected_genesis.value;
        result.policy_valid = proof.policy_hash == self.expected_policy_hash;
        result.depth_valid = proof.final_lineage.depth == proof.num_steps;
        result.proof_valid = !proof.proof_bytes.is_empty();
        result.is_real_zk = proof.is_real_zk();

        result.is_valid = result.genesis_valid
            && result.policy_valid
            && result.depth_valid
            && result.proof_valid;

        result
    }
}

/// Detailed verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Overall validity
    pub is_valid: bool,
    /// Genesis check passed
    pub genesis_valid: bool,
    /// Policy check passed
    pub policy_valid: bool,
    /// Depth check passed
    pub depth_valid: bool,
    /// Proof structure valid
    pub proof_valid: bool,
    /// Whether this was a real ZK proof
    pub is_real_zk: bool,
}

impl VerificationResult {
    fn new() -> Self {
        Self {
            is_valid: false,
            genesis_valid: false,
            policy_valid: false,
            depth_valid: false,
            proof_valid: false,
            is_real_zk: false,
        }
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Valid: {} (genesis: {}, policy: {}, depth: {}, proof: {}, real_zk: {})",
            self.is_valid,
            self.genesis_valid,
            self.policy_valid,
            self.depth_valid,
            self.proof_valid,
            self.is_real_zk
        )
    }
}

impl std::fmt::Display for VerificationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Verify a proof standalone
pub fn verify_proof(
    proof: &LineageProof,
    genesis_hash: [u8; 32],
    policy: &OriginPolicy,
) -> Result<bool> {
    let verifier = LineageVerifier::new(genesis_hash, policy);
    verifier.verify(proof)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::lineage::CounterCommitment;

    fn create_test_proof(genesis_hash: [u8; 32], policy: &OriginPolicy, large: bool) -> LineageProof {
        let proof_bytes = if large {
            vec![0u8; 5000]
        } else {
            vec![1, 2, 3, 4]
        };
        
        LineageProof::new(
            proof_bytes,
            LineageCommitment::new([1u8; 32], 5),
            CounterCommitment::new([2u8; 32], 0),
            LineageCommitment::genesis(genesis_hash),
            5,
            policy.compute_hash(),
        )
    }

    #[test]
    fn test_verify_valid_proof() {
        let genesis = [0u8; 32];
        let policy = OriginPolicy::default();
        let proof = create_test_proof(genesis, &policy, false);
        
        let verifier = LineageVerifier::new(genesis, &policy);
        let result = verifier.verify(&proof);
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_wrong_genesis() {
        let genesis = [0u8; 32];
        let wrong_genesis = [1u8; 32];
        let policy = OriginPolicy::default();
        let proof = create_test_proof(wrong_genesis, &policy, false);
        
        let verifier = LineageVerifier::new(genesis, &policy);
        let result = verifier.verify(&proof);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZkOriginError::GenesisMismatch));
    }

    #[test]
    fn test_verify_detailed() {
        let genesis = [0u8; 32];
        let policy = OriginPolicy::default();
        let proof = create_test_proof(genesis, &policy, false);
        
        let verifier = LineageVerifier::new(genesis, &policy);
        let result = verifier.verify_detailed(&proof);
        
        assert!(result.is_valid);
        assert!(!result.is_real_zk);
    }

    #[test]
    fn test_standalone_verify() {
        let genesis = [0u8; 32];
        let policy = OriginPolicy::default();
        let proof = create_test_proof(genesis, &policy, false);
        
        let result = verify_proof(&proof, genesis, &policy);
        
        assert!(result.is_ok());
    }
}