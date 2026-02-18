//! Public input handling for verification

use crate::types::{LineageCommitment, CounterCommitment};
use serde::{Deserialize, Serialize};

/// Public inputs for verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicInputs {
    /// Genesis lineage commitment
    pub genesis_lineage: [u8; 32],
    
    /// Final lineage commitment
    pub final_lineage: [u8; 32],
    
    /// Final counter commitment
    pub final_counters: [u8; 32],
    
    /// Number of steps
    pub num_steps: u64,
    
    /// Policy hash
    pub policy_hash: [u8; 32],
}

impl PublicInputs {
    /// Create from components
    pub fn new(
        genesis_lineage: [u8; 32],
        final_lineage: [u8; 32],
        final_counters: [u8; 32],
        num_steps: u64,
        policy_hash: [u8; 32],
    ) -> Self {
        Self {
            genesis_lineage,
            final_lineage,
            final_counters,
            num_steps,
            policy_hash,
        }
    }

    /// Create from proof
    pub fn from_proof(proof: &crate::types::LineageProof) -> Self {
        Self {
            genesis_lineage: proof.genesis_commitment.value,
            final_lineage: proof.final_lineage.value,
            final_counters: proof.final_counters.value,
            num_steps: proof.num_steps,
            policy_hash: proof.policy_hash,
        }
    }

    /// Serialize to bytes for on-chain verification
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(136);
        bytes.extend_from_slice(&self.genesis_lineage);
        bytes.extend_from_slice(&self.final_lineage);
        bytes.extend_from_slice(&self.final_counters);
        bytes.extend_from_slice(&self.num_steps.to_le_bytes());
        bytes.extend_from_slice(&self.policy_hash);
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 136 {
            return None;
        }

        let mut genesis_lineage = [0u8; 32];
        let mut final_lineage = [0u8; 32];
        let mut final_counters = [0u8; 32];
        let mut num_steps_bytes = [0u8; 8];
        let mut policy_hash = [0u8; 32];

        genesis_lineage.copy_from_slice(&bytes[0..32]);
        final_lineage.copy_from_slice(&bytes[32..64]);
        final_counters.copy_from_slice(&bytes[64..96]);
        num_steps_bytes.copy_from_slice(&bytes[96..104]);
        policy_hash.copy_from_slice(&bytes[104..136]);

        Some(Self {
            genesis_lineage,
            final_lineage,
            final_counters,
            num_steps: u64::from_le_bytes(num_steps_bytes),
            policy_hash,
        })
    }

    /// Get as hex-encoded string (for debugging)
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_inputs_serialization() {
        let inputs = PublicInputs::new(
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
            42,
            [4u8; 32],
        );

        let bytes = inputs.to_bytes();
        let recovered = PublicInputs::from_bytes(&bytes).unwrap();

        assert_eq!(inputs.genesis_lineage, recovered.genesis_lineage);
        assert_eq!(inputs.final_lineage, recovered.final_lineage);
        assert_eq!(inputs.num_steps, recovered.num_steps);
    }

    #[test]
    fn test_from_bytes_too_short() {
        let bytes = vec![0u8; 50];
        assert!(PublicInputs::from_bytes(&bytes).is_none());
    }
}