//! Proof verification module

use crate::error::Result;
use crate::nova_ivc::CompressedNovaProof;

/// Proof statistics
#[derive(Clone, Debug)]
pub struct ProofStats {
    /// Total proof size in bytes
    pub size_bytes: usize,
    /// Number of steps in proof
    pub steps: usize,
    /// Average size per step
    pub avg_step_size: usize,
    /// Compression ratio
    pub compression_ratio: f64,
}

/// Proof verifier for Nova IVC proofs
pub struct NovaVerifier;

impl NovaVerifier {
    /// Verify a proof offline
    pub fn verify(
        proof: &CompressedNovaProof,
        genesis_state: &[u8],
        final_state: &[u8],
    ) -> Result<bool> {
        // Validate proof structure
        proof.validate()?;

        // Verify genesis matches
        if proof.genesis_state != genesis_state {
            return Ok(false);
        }

        // Verify final state matches
        if proof.final_state != final_state {
            return Ok(false);
        }

        // Verify steps > 0
        if proof.steps == 0 {
            return Ok(false);
        }

        // Verify proof data exists
        if proof.proof_data.is_empty() {
            return Ok(false);
        }

        // Verify checksum matches (tampering detection)
        let expected_checksum = proof.compute_checksum();
        if expected_checksum != proof.checksum {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get proof statistics
    pub fn get_stats(proof: &CompressedNovaProof) -> ProofStats {
        let size = proof.size_bytes();
        ProofStats {
            size_bytes: size,
            steps: proof.steps,
            avg_step_size: size / proof.steps.max(1),
            compression_ratio: proof.compression_ratio(),
        }
    }

    /// Batch verify multiple proofs
    pub fn verify_batch(proofs: &[CompressedNovaProof], genesis_state: &[u8]) -> Result<Vec<bool>> {
        proofs
            .iter()
            .map(|proof| {
                let final_state = &proof.final_state;
                Self::verify(proof, genesis_state, final_state)
            })
            .collect()
    }

    /// Verify proof chain (each proof's final state is next proof's genesis)
    pub fn verify_chain(proofs: &[CompressedNovaProof]) -> Result<bool> {
        if proofs.is_empty() {
            return Ok(false);
        }

        for i in 0..proofs.len() - 1 {
            let current_final = &proofs[i].final_state;
            let next_genesis = &proofs[i + 1].genesis_state;

            if current_final != next_genesis {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_valid_proof() {
        let genesis = vec![0u8; 48];
        let final_state = vec![1u8; 48];

        let mut proof = CompressedNovaProof {
            proof_data: vec![0u8; 2500],
            final_state: final_state.clone(),
            steps: 10,
            genesis_state: genesis.clone(),
            timestamp: 0,
            circuit_hash: Default::default(),
            proof_commitment: Default::default(),
            checksum: Default::default(),
        };

        proof.checksum = proof.compute_checksum();

        let verified = NovaVerifier::verify(&proof, &genesis, &final_state);
        assert!(verified.is_ok());
        assert!(verified.unwrap());
    }

    #[test]
    fn test_verify_chain() {
        let mut proof1 = CompressedNovaProof {
            proof_data: vec![0u8; 2500],
            final_state: vec![1u8; 48],
            steps: 5,
            genesis_state: vec![0u8; 48],
            timestamp: 0,
            circuit_hash: Default::default(),
            proof_commitment: Default::default(),
            checksum: Default::default(),
        };

        let mut proof2 = CompressedNovaProof {
            proof_data: vec![0u8; 2500],
            final_state: vec![2u8; 48],
            steps: 5,
            genesis_state: vec![1u8; 48], // Matches proof1's final
            timestamp: 0,
            circuit_hash: Default::default(),
            proof_commitment: Default::default(),
            checksum: Default::default(),
        };

        proof1.checksum = proof1.compute_checksum();
        proof2.checksum = proof2.compute_checksum();

        let verified = NovaVerifier::verify_chain(&[proof1, proof2]);
        assert!(verified.is_ok());
        assert!(verified.unwrap());
    }

    #[test]
    fn test_get_stats() {
        let proof = CompressedNovaProof {
            proof_data: vec![0u8; 2500],
            final_state: vec![0u8; 48],
            steps: 10,
            genesis_state: vec![0u8; 48],
            timestamp: 0,
            circuit_hash: Default::default(),
            proof_commitment: Default::default(),
            checksum: Default::default(),
        };

        let stats = NovaVerifier::get_stats(&proof);
        assert!(stats.size_bytes > 0);
        assert_eq!(stats.steps, 10);
    }
}