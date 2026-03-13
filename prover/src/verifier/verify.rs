//! Proof verification implementation

use crate::types::lineage::LineageCommitment;
use crate::types::proof::LineageProof;
use crate::types::OriginPolicy;
use crate::{Result, ZkOriginError};
use std::time::Instant;

#[cfg(feature = "real-nova")]
use {
    ff::{Field, PrimeField},
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
}

impl LineageVerifier {
    /// Create a new verifier with a genesis state hash
    pub fn new(genesis_state_hash: [u8; 32], policy: &OriginPolicy) -> Self {
        Self {
            expected_genesis: LineageCommitment::genesis(genesis_state_hash),
            expected_policy_hash: policy.compute_hash(),
            policy: policy.clone(),
        }
    }

    /// Create a verifier from the proof's genesis commitment
    pub fn from_proof(proof: &LineageProof, policy: &OriginPolicy) -> Self {
        Self {
            expected_genesis: proof.genesis_commitment.clone(),
            expected_policy_hash: policy.compute_hash(),
            policy: policy.clone(),
        }
    }

    /// Create verifier with a specific genesis commitment
    pub fn with_genesis_commitment(genesis: LineageCommitment, policy: &OriginPolicy) -> Self {
        Self {
            expected_genesis: genesis,
            expected_policy_hash: policy.compute_hash(),
            policy: policy.clone(),
        }
    }

    /// Create verifier from proof with Nova params (for ZK verification)
    #[cfg(feature = "real-nova")]
    pub fn from_proof_with_nova(
        proof: &LineageProof,
        policy: &OriginPolicy,
        _nova_params: &'static crate::prover::NovaParams,
    ) -> Self {
        Self::from_proof(proof, policy)
    }

    /// Get the expected genesis commitment
    pub fn expected_genesis(&self) -> &LineageCommitment {
        &self.expected_genesis
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

        let vk_bytes = proof.verifier_key.as_ref().ok_or_else(|| {
            ZkOriginError::InvalidProof("Missing verifier key for ZK proof".into())
        })?;

        // Use the Nova step count stored in the proof
        let nova_steps = proof.get_nova_steps();

        self.verify_compressed_snark(proof, vk_bytes, nova_steps)
    }

    /// Verify compressed SNARK cryptographically
    #[cfg(feature = "real-nova")]
    fn verify_compressed_snark(
        &self,
        proof: &LineageProof,
        vk_bytes: &[u8],
        nova_steps: u64,
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

        // Deserialize verifier key
        println!("  Deserializing verifier key...");
        let vk_deser_start = Instant::now();

        if vk_bytes.is_empty() {
            return Err(ZkOriginError::InvalidProof("Empty verifier key".into()));
        }

        let vk: nova_snark::VerifierKey<G1, G2, C1, C2, S1, S2> =
            bincode::deserialize(vk_bytes).map_err(|e| {
                ZkOriginError::proving(format!("Failed to deserialize VK: {}", e))
            })?;

        println!("    Deserialized in {:?}", vk_deser_start.elapsed());

        // Reconstruct initial input from the proof's genesis commitment
        println!("  Reconstructing public inputs...");
        let genesis_f = bytes_to_field::<F1>(&proof.genesis_commitment.value);
        let z0_primary = vec![genesis_f, F1::ZERO];
        let z0_secondary = vec![F2::ZERO];

        println!(
            "  Verifying compressed SNARK ({} Nova steps, {} logical steps)...",
            nova_steps, proof.num_steps
        );
        let verify_start = Instant::now();

        // Verify the compressed SNARK with the correct Nova step count
        compressed
            .verify(&vk, nova_steps as usize, &z0_primary, &z0_secondary)
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

        #[cfg(feature = "real-nova")]
        if result.is_real_zk {
            match self.verify_zk(proof) {
                Ok(true) => {
                    result.cryptographic_verified = true;
                }
                Ok(false) => {
                    result.is_valid = false;
                    result.cryptographic_verified = false;
                }
                Err(e) => {
                    result.is_valid = false;
                    result.cryptographic_verified = false;
                    result.error_message = Some(format!("{}", e));
                }
            }
        }

        result
    }
}

/// Detailed verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub genesis_valid: bool,
    pub policy_valid: bool,
    pub depth_valid: bool,
    pub proof_valid: bool,
    pub is_real_zk: bool,
    pub cryptographic_verified: bool,
    pub error_message: Option<String>,
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
            error_message: None,
        }
    }

    pub fn summary(&self) -> String {
        let base = format!(
            "Valid: {} (genesis: {}, policy: {}, depth: {}, proof: {}, real_zk: {}, crypto: {})",
            self.is_valid,
            self.genesis_valid,
            self.policy_valid,
            self.depth_valid,
            self.proof_valid,
            self.is_real_zk,
            self.cryptographic_verified
        );

        if let Some(ref err) = self.error_message {
            format!("{} [Error: {}]", base, err)
        } else {
            base
        }
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

/// Verify a proof using the genesis from the proof itself
pub fn verify_proof_self_consistent(proof: &LineageProof, policy: &OriginPolicy) -> Result<bool> {
    let verifier = LineageVerifier::from_proof(proof, policy);
    verifier.verify(proof)
}

/// Verify a real ZK proof (requires Nova feature)
#[cfg(feature = "real-nova")]
pub fn verify_zk_proof(
    proof: &LineageProof,
    genesis_hash: [u8; 32],
    policy: &OriginPolicy,
) -> Result<bool> {
    let verifier = LineageVerifier::new(genesis_hash, policy);
    verifier.verify_zk(proof)
}

/// Verify a real ZK proof using genesis from proof
#[cfg(feature = "real-nova")]
pub fn verify_zk_proof_self_consistent(proof: &LineageProof, policy: &OriginPolicy) -> Result<bool> {
    let verifier = LineageVerifier::from_proof(proof, policy);
    verifier.verify_zk(proof)
}

#[cfg(feature = "real-nova")]
fn bytes_to_field<F: PrimeField>(bytes: &[u8; 32]) -> F {
    let mut repr = F::Repr::default();
    let repr_len = repr.as_ref().len();
    let copy_len = std::cmp::min(repr_len, 31);
    repr.as_mut()[..copy_len].copy_from_slice(&bytes[..copy_len]);
    F::from_repr(repr).unwrap_or(F::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::lineage::CounterCommitment;

    fn create_test_proof(genesis_hash: [u8; 32], policy: &OriginPolicy, large: bool) -> LineageProof {
        let proof_bytes = if large { vec![0u8; 5000] } else { vec![1, 2, 3, 4] };

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
    fn test_verify_from_proof() {
        let genesis = [42u8; 32];
        let policy = OriginPolicy::default();
        let proof = create_test_proof(genesis, &policy, false);

        let verifier = LineageVerifier::from_proof(&proof, &policy);
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
    fn test_self_consistent_verify() {
        let genesis = [123u8; 32];
        let policy = OriginPolicy::default();
        let proof = create_test_proof(genesis, &policy, false);

        let result = verify_proof_self_consistent(&proof, &policy);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}