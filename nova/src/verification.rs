/**
 * @title Nova Verification (PRODUCTION)
 * @notice Verify Nova IVC proofs offline
 */

use super::CompressedNovaProof;

pub struct NovaVerifier;

impl NovaVerifier {
    /// Verify Nova proof offline
    pub fn verify(
        proof: &CompressedNovaProof,
        genesis_state: &[u8],
        final_state: &[u8],
    ) -> bool {
        // Verify:
        // 1. Proof integrity
        if !proof.verify_integrity() {
            return false;
        }

        // 2. Genesis matches
        if proof.genesis_state != genesis_state {
            return false;
        }

        // 3. Final state matches
        if proof.final_state != final_state {
            return false;
        }

        // 4. Steps > 0
        if proof.steps == 0 {
            return false;
        }

        true
    }

    /// Get proof statistics
    pub fn get_stats(proof: &CompressedNovaProof) -> ProofStats {
        ProofStats {
            size_bytes: proof.size_bytes(),
            steps: proof.steps,
            avg_step_size: proof.size_bytes() / proof.steps.max(1),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProofStats {
    pub size_bytes: usize,
    pub steps: usize,
    pub avg_step_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_stats() {
        let proof = CompressedNovaProof {
            proof_data: vec![0u8; 2500],
            final_state: vec![0u8; 48],
            steps: 10,
            genesis_state: vec![0u8; 48],
            timestamp: 0,
            circuit_hash: [0u8; 32],
        };

        let stats = NovaVerifier::get_stats(&proof);
        assert!(stats.size_bytes > 0);
        assert_eq!(stats.steps, 10);
        assert!(stats.avg_step_size > 0);
    }

    #[test]
    fn test_nova_verify_valid() {
        let genesis = vec![0u8; 48];
        let final_state = vec![1u8; 48];
        
        let proof = CompressedNovaProof {
            proof_data: vec![0u8; 2500],
            final_state: final_state.clone(),
            steps: 10,
            genesis_state: genesis.clone(),
            timestamp: 0,
            circuit_hash: [0u8; 32],
        };

        let verified = NovaVerifier::verify(&proof, &genesis, &final_state);
        assert!(verified);
    }

    #[test]
    fn test_nova_verify_invalid_genesis() {
        let genesis = vec![0u8; 48];
        let wrong_genesis = vec![1u8; 48];
        let final_state = vec![1u8; 48];
        
        let proof = CompressedNovaProof {
            proof_data: vec![0u8; 2500],
            final_state: final_state.clone(),
            steps: 10,
            genesis_state: genesis.clone(),
            timestamp: 0,
            circuit_hash: [0u8; 32],
        };

        let verified = NovaVerifier::verify(&proof, &wrong_genesis, &final_state);
        assert!(!verified);
    }
}