//! Witness validation

use crate::error::{ProverError, Result};
use crate::witness::generator::TransitionWitness;

/// Witness validator
pub struct WitnessValidator;

impl WitnessValidator {
    /// Validate witness completely
    pub fn validate(witness: &TransitionWitness) -> Result<()> {
        // Use witness's own validation
        witness.validate()
    }

    /// Validate batch of witnesses
    pub fn validate_batch(witnesses: &[TransitionWitness]) -> Result<()> {
        for (idx, witness) in witnesses.iter().enumerate() {
            Self::validate(witness).map_err(|e| {
                ProverError::batch_operation_failed(format!(
                    "Witness {} validation failed: {}",
                    idx, e
                ))
            })?;
        }
        Ok(())
    }

    /// Check witness consistency with previous
    pub fn validate_sequence(witnesses: &[TransitionWitness]) -> Result<()> {
        for i in 1..witnesses.len() {
            let prev = &witnesses[i - 1];
            let curr = &witnesses[i];

            // Check lineage continuity
            if prev.public.new_state_hash != curr.public.prev_state_hash {
                return Err(ProverError::invalid_state(format!(
                    "State mismatch at position {}: {} != {}",
                    i, prev.public.new_state_hash, curr.public.prev_state_hash
                )));
            }

            // Check nonce continuity
            if curr.private.prev_nonce != prev.private.nonce {
                return Err(ProverError::invalid_nonce(format!(
                    "Nonce mismatch at position {}",
                    i
                )));
            }

            // Check timestamp progression
            if curr.private.prev_timestamp != prev.private.timestamp {
                return Err(ProverError::invalid_timestamp(format!(
                    "Timestamp mismatch at position {}",
                    i
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::witness::generator::{PrivateWitness, PublicWitness};

    #[test]
    fn test_batch_validation() {
        let witnesses = vec![TransitionWitness {
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
        }];

        assert!(WitnessValidator::validate_batch(&witnesses).is_ok());
    }
}
