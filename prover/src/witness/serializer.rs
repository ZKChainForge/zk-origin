// prover/src/witness/serializer.rs

use crate::error::Result;
use crate::witness::generator::TransitionWitness;
use std::fs;

/// Witness serializer
pub struct WitnessSerializer;

impl WitnessSerializer {
    /// Serialize witness to Circom-compatible JSON
    pub fn to_circom_json(witness: &TransitionWitness) -> Result<serde_json::Value> {
        witness.to_json()
    }

    /// Serialize to file
    pub fn to_file(witness: &TransitionWitness, path: &str) -> Result<()> {
        let json = Self::to_circom_json(witness)?;
        let content = serde_json::to_string_pretty(&json)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Deserialize from file
    pub fn from_file(path: &str) -> Result<TransitionWitness> {
        let content = fs::read_to_string(path)?;
        let witness: TransitionWitness = serde_json::from_str(&content)?;
        witness.validate()?;
        Ok(witness)
    }

    /// Get witness summary (for logging)
    pub fn summary(witness: &TransitionWitness) -> String {
        format!(
            "Witness(nonce: {}, epoch: {}, from_class: {}, to_class: {})",
            witness.private.nonce,
            witness.private.prev_epoch_id,
            witness.public.prev_origin_class,
            witness.public.new_origin_class,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::witness::generator::{PrivateWitness, PublicWitness};

    #[test]
    fn test_json_serialization() {
        let witness = TransitionWitness {
            public: PublicWitness {
                new_lineage_commitment: "1".to_string(),
                new_counter_commitment: "1".to_string(),
                lineage_valid: 1,
                prev_state_hash: "1".to_string(),
                new_state_hash: "2".to_string(),
                epoch_id: 0,
                prev_origin_class: 0,
                new_origin_class: 1,
                prev_lineage_commitment: "0".to_string(),
                prev_counter_commitment: "0".to_string(),
                policy_root: "0".to_string(),
                expected_genesis_hash: "0".to_string(),
            },
            private: PrivateWitness {
                prev_epoch_id: 0,
                prev_depth: 0,
                nonce: 1,
                prev_nonce: 0,
                timestamp: 1000,
                prev_timestamp: 999,
                policy_proof: vec![],
                policy_indices: vec![],
                prev_counters: vec![0, 0, 0, 0, 0, 0, 0],
                rate_limits: vec![1, u32::MAX, 10, 100, 5, 1000, 1],
                public_key_x: None,
                public_key_y: None,
                signature_r: None,
                signature_s: None,
                authorization_valid: 1,
            },
        };

        let json = WitnessSerializer::to_circom_json(&witness);
        assert!(json.is_ok());
    }

    #[test]
    fn test_summary() {
        let witness = TransitionWitness {
            public: PublicWitness {
                new_lineage_commitment: "1".to_string(),
                new_counter_commitment: "1".to_string(),
                lineage_valid: 1,
                prev_state_hash: "1".to_string(),
                new_state_hash: "2".to_string(),
                epoch_id: 0,
                prev_origin_class: 0,
                new_origin_class: 1,
                prev_lineage_commitment: "0".to_string(),
                prev_counter_commitment: "0".to_string(),
                policy_root: "0".to_string(),
                expected_genesis_hash: "0".to_string(),
            },
            private: PrivateWitness {
                prev_epoch_id: 0,
                prev_depth: 0,
                nonce: 1,
                prev_nonce: 0,
                timestamp: 1000,
                prev_timestamp: 999,
                policy_proof: vec![],
                policy_indices: vec![],
                prev_counters: vec![0, 0, 0, 0, 0, 0, 0],
                rate_limits: vec![1, u32::MAX, 10, 100, 5, 1000, 1],
                public_key_x: None,
                public_key_y: None,
                signature_r: None,
                signature_s: None,
                authorization_valid: 1,
            },
        };

        let summary = WitnessSerializer::summary(&witness);
        assert!(summary.contains("nonce: 1"));
    }
}