//! Proof verification implementation

use crate::types::lineage::LineageCommitment;
use crate::types::proof::LineageProof;
use crate::types::OriginPolicy;
use crate::{Result, ZkOriginError};
use std::time::Instant;

#[cfg(feature = "real-nova")]
use {
    crate::prover::NovaParams,
    ff::PrimeField,
    nova_snark::CompressedSNARK,
    pasta_curves::pallas,
};

/// Verifier for lineage proofs
pub struct LineageVerifier {
    /// Expected genesis commitment
    expected_genesis: LineageCommitment,

    /// Expected policy hash
    expected_policy_hash: [u8; 32],

    /// Policy (for reference)
    #[allow(dead_code)]
    policy: OriginPolicy,

    /// Nova parameters (for real ZK verification)
    #[cfg(feature = "real-nova")]
    nova_params: Option<&'static NovaParams>,
}

impl LineageVerifier {
    /// Create a new verifier
    pub fn new(genesis_state_hash: [u8; 32], policy: &OriginPolicy) -> Self {
        Self {
            expected_genesis: LineageCommitment::genesis(genesis_state_hash),
            expected_policy_hash: policy.compute_hash(),
            policy: policy.clone(),
            #[cfg(feature = "real-nova")]
            nova_params: None,
        }
    }

    /// Create verifier with Nova parameters for real ZK verification
    #[cfg(feature = "real-nova")]
    pub fn with_nova_params(
        genesis_state_hash: [u8; 32],
        policy: &OriginPolicy,
        nova_params: &'static NovaParams,
    ) -> Self {
        Self {
            expected_genesis: LineageCommitment::genesis(genesis_state_hash),
            expected_policy_hash: policy.compute_hash(),
            policy: policy.clone(),
            nova_params: Some(nova_params),
        }
    }

    /// Verify a lineage proof (structural checks only)
    pub fn verify(&self, proof: &LineageProof) -> Result<bool> {
        // Check 1: Genesis matches
        if proof.genesis_commitment.value != self.expected_genesis.value {
            return Err(ZkOriginError::GenesisMismatch);
        }

        // Check 2: Policy matches
        if proof.policy_hash != self.expected_policy_hash {
            return Err(ZkOriginError::VerificationFailed(
                "Policy hash mismatch".into(),
            ));
        }

        // Check 3: Depth consistency
        if proof.final_lineage.depth != proof.num_steps {
            return Err(ZkOriginError::VerificationFailed("Depth mismatch".into()));
        }

        // Check 4: Proof non-empty
        if proof.proof_bytes.is_empty() {
            return Err(ZkOriginError::InvalidProof("Empty proof".into()));
        }

        Ok(true)
    }

    /// Verify a real ZK proof cryptographically (requires Nova feature)
    #[cfg(feature = "real-nova")]
    pub fn verify_zk(&self, proof: &LineageProof) -> Result<bool> {
        // First do structural checks
        self.verify(proof)?;

        let _params = self
            .nova_params
            .ok_or_else(|| ZkOriginError::NotInitialized(
                "Nova parameters not provided for ZK verification".into(),
            ))?;

        let vk_bytes = proof.verifier_key.as_ref().ok_or_else(|| {
            ZkOriginError::InvalidProof("Missing verifier key for ZK proof".into())
        })?;

        self.verify_compressed_snark(proof, vk_bytes)
    }

    /// Verify compressed SNARK cryptographically
    #[cfg(feature = "real-nova")]
    fn verify_compressed_snark(
        &self,
        proof: &LineageProof,
        vk_bytes: &[u8],
    ) -> Result<bool> {
        use crate::prover::nova_circuit::LineageStepCircuit;
        use pasta_curves::vesta;

        type G1 = pallas::Point;
        type G2 = vesta::Point;
        type F1 = pallas::Scalar;
        type F2 = vesta::Scalar;
        type EE1 = nova_snark::provider::ipa_pc::EvaluationEngine<G1>;
        type EE2 = nova_snark::provider::ipa_pc::EvaluationEngine<G2>;
        type S1 = nova_snark::spartan::snark::RelaxedR1CSSNARK<G1, EE1>;
        type S2 = nova_snark::spartan::snark::RelaxedR1CSSNARK<G2, EE2>;
        type C1 = LineageStepCircuit<F1>;
        type C2 = nova_snark::traits::circuit::TrivialCircuit<F2>;

        println!("═ Real ZK Proof Verification");
        println!("  Deserializing compressed proof...");
        let deser_start = Instant::now();

        // Deserialize proof
        let compressed: CompressedSNARK<G1, G2, C1, C2, S1, S2> =
            bincode::deserialize(&proof.proof_bytes).map_err(|e| {
                ZkOriginError::proving(format!("Failed to deserialize proof: {}", e))
            })?;

        println!("    Deserialized in {:?}", deser_start.elapsed());

        // Validate verifier key
        println!("  Validating verifier key...");
        let vk_deser_start = Instant::now();

        if vk_bytes.is_empty() {
            return Err(ZkOriginError::InvalidProof(
                "Empty verifier key".into(),
            ));
        }

        println!("    Validated in {:?}", vk_deser_start.elapsed());

        // Reconstruct initial input from genesis
        println!("  Reconstructing public inputs...");
        let genesis_f = bytes_to_field::<F1>(&proof.genesis_commitment.value);
        let z0_primary = vec![genesis_f, create_zero::<F1>()];
        let z0_secondary = vec![create_zero::<F2>()];

        // Perform cryptographic verification
        println!(
            "  Verifying compressed SNARK ({} steps)...",
            proof.num_steps
        );
        let verify_start = Instant::now();

        // Verify - CompressedSNARK requires vk as first parameter
        compressed
            .verify(
                &bincode::deserialize::<
                    nova_snark::VerifierKey<G1, G2, C1, C2, S1, S2>,
                >(vk_bytes)
                .map_err(|e| {
                    ZkOriginError::proving(format!("Failed to deserialize VK: {}", e))
                })?,
                proof.num_steps as usize,
                &z0_primary,
                &z0_secondary,
            )
            .map_err(|e| {
                ZkOriginError::VerificationFailed(format!(
                    "SNARK verification failed: {:?}",
                    e
                ))
            })?;

        let verify_time = verify_start.elapsed();
        println!("  ✓ Verification passed in {:?}", verify_time);

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

        result.is_valid =
            result.genesis_valid && result.policy_valid && result.depth_valid && result.proof_valid;

        // Try cryptographic verification if available
        #[cfg(feature = "real-nova")]
        if result.is_real_zk && self.nova_params.is_some() {
            match self.verify_zk(proof) {
                Ok(true) => {
                    result.cryptographic_verified = true;
                }
                Ok(false) => {
                    result.is_valid = false;
                    result.cryptographic_verified = false;
                }
                Err(_) => {
                    result.is_valid = false;
                    result.cryptographic_verified = false;
                }
            }
        }

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
    /// Whether cryptographic verification was performed and passed
    pub cryptographic_verified: bool,
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
            cryptographic_verified: false,
        }
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Valid: {} (genesis: {}, policy: {}, depth: {}, proof: {}, real_zk: {}, crypto_verified: {})",
            self.is_valid,
            self.genesis_valid,
            self.policy_valid,
            self.depth_valid,
            self.proof_valid,
            self.is_real_zk,
            self.cryptographic_verified
        )
    }
}

impl std::fmt::Display for VerificationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Verify a proof standalone (structural only)
pub fn verify_proof(
    proof: &LineageProof,
    genesis_hash: [u8; 32],
    policy: &OriginPolicy,
) -> Result<bool> {
    let verifier = LineageVerifier::new(genesis_hash, policy);
    verifier.verify(proof)
}

/// Verify a real ZK proof (requires Nova feature)
#[cfg(feature = "real-nova")]
pub fn verify_zk_proof(
    proof: &LineageProof,
    genesis_hash: [u8; 32],
    policy: &OriginPolicy,
    nova_params: &'static NovaParams,
) -> Result<bool> {
    let verifier = LineageVerifier::with_nova_params(genesis_hash, policy, nova_params);
    verifier.verify_zk(proof)
}

/// Create a zero field element (works without num_traits)
#[cfg(feature = "real-nova")]
fn create_zero<F: ff::PrimeField>() -> F {
    let repr = F::Repr::default();
    F::from_repr(repr).unwrap()
}

/// Convert bytes to field element
#[cfg(feature = "real-nova")]
fn bytes_to_field<F: ff::PrimeField>(bytes: &[u8; 32]) -> F {
    let mut repr = F::Repr::default();
    let repr_slice = repr.as_mut();
    
    let copy_len = std::cmp::min(repr_slice.len(), bytes.len());
    repr_slice[..copy_len].copy_from_slice(&bytes[..copy_len]);
    
    F::from_repr(repr).unwrap_or_else(|| {
        // Return zero if conversion fails
        let zero_repr = F::Repr::default();
        F::from_repr(zero_repr).unwrap()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::lineage::CounterCommitment;

    fn create_test_proof(
        genesis_hash: [u8; 32],
        policy: &OriginPolicy,
        large: bool,
    ) -> LineageProof {
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
        assert!(matches!(
            result.unwrap_err(),
            ZkOriginError::GenesisMismatch
        ));
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