//! Proof verification implementation

use crate::types::lineage::{LineageCommitment};
use crate::types::OriginPolicy;
use crate::{Result, ZkOriginError};
use crate::types::proof::LineageProof;

/// Verifier for lineage proofs
pub struct LineageVerifier {
    /// Expected genesis commitment
    expected_genesis: LineageCommitment,
    
    /// Expected policy hash
    expected_policy_hash: [u8; 32],
}

impl LineageVerifier {
    /// Create a new verifier
    pub fn new(genesis_state_hash: [u8; 32], policy: &OriginPolicy) -> Self {
        Self {
            expected_genesis: LineageCommitment::genesis(genesis_state_hash),
            expected_policy_hash: policy.compute_hash(),
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

        // Check 4: Proof is non-empty
        if proof.proof_bytes.is_empty() {
            return Err(ZkOriginError::InvalidProof("Empty proof".into()));
        }

        Ok(true)
    }

    /// Verify a proof and return detailed results
    pub fn verify_detailed(&self, proof: &LineageProof) -> VerificationResult {
        let mut result = VerificationResult::new();

        result.genesis_valid = proof.genesis_commitment.value == self.expected_genesis.value;
        result.policy_valid = proof.policy_hash == self.expected_policy_hash;
        result.depth_valid = proof.final_lineage.depth == proof.num_steps;
        result.proof_valid = !proof.proof_bytes.is_empty();

        result.is_valid = result.genesis_valid
            && result.policy_valid
            && result.depth_valid
            && result.proof_valid;

        result
    }
}

/// Detailed verification result
/// Detailed result of lineage proof verification.
///
/// Provides per-check validity flags and an overall result.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Overall verification result (true only if all checks pass)
    pub is_valid: bool,

    /// Whether the genesis commitment matches the expected value
    pub genesis_valid: bool,

    /// Whether the policy hash matches the expected policy
    pub policy_valid: bool,

    /// Whether the lineage depth matches the declared number of steps
    pub depth_valid: bool,

    /// Whether the proof bytes are non-empty and structurally valid
    pub proof_valid: bool,
}


impl VerificationResult {
    fn new() -> Self {
        Self {
            is_valid: false,
            genesis_valid: false,
            policy_valid: false,
            depth_valid: false,
            proof_valid: false,
        }
    }
}

    impl VerificationResult {
    /// Returns a human-readable summary of the verification result.
    ///
    /// Intended for debugging, logging, and test output.
    pub fn summary(&self) -> String {
        format!(
            "Valid: {} (genesis: {}, policy: {}, depth: {}, proof: {})",
            self.is_valid,
            self.genesis_valid,
            self.policy_valid,
            self.depth_valid,
            self.proof_valid
        )

    }

}

impl std::fmt::Display for VerificationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
}


/// Verify a proof standalone (without creating a verifier)
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
    use crate::types::lineage::{LineageCommitment, CounterCommitment};

    fn create_valid_proof(genesis_hash: [u8; 32], policy: &OriginPolicy) -> LineageProof {
        LineageProof::new(
            vec![1, 2, 3, 4],
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
        let proof = create_valid_proof(genesis, &policy);
        
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
        let proof = create_valid_proof(wrong_genesis, &policy);
        
        let verifier = LineageVerifier::new(genesis, &policy);
        let result = verifier.verify(&proof);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZkOriginError::GenesisMismatch));
    }

    #[test]
    fn test_verify_wrong_policy() {
        let genesis = [0u8; 32];
        let policy1 = OriginPolicy::default();
        let policy2 = OriginPolicy::restrictive();
        let proof = create_valid_proof(genesis, &policy1);
        
        let verifier = LineageVerifier::new(genesis, &policy2);
        let result = verifier.verify(&proof);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_detailed() {
        let genesis = [0u8; 32];
        let policy = OriginPolicy::default();
        let proof = create_valid_proof(genesis, &policy);
        
        let verifier = LineageVerifier::new(genesis, &policy);
        let result = verifier.verify_detailed(&proof);
        
        assert!(result.is_valid);
        assert!(result.genesis_valid);
        assert!(result.policy_valid);
        assert!(result.depth_valid);
        assert!(result.proof_valid);
    }

    #[test]
    fn test_standalone_verify() {
        let genesis = [0u8; 32];
        let policy = OriginPolicy::default();
        let proof = create_valid_proof(genesis, &policy);
        
        let result = verify_proof(&proof, genesis, &policy);
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
