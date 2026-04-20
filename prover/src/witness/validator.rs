//! Witness validation

use crate::{error::Result, witness::generator::TransitionWitness};

/// Witness validator
pub struct WitnessValidator;

impl WitnessValidator {
    /// Validate witness
    pub fn validate(witness: &TransitionWitness) -> Result<()> {
        // Validate nonce
        if witness.private.nonce <= witness.private.prev_nonce {
            return Err(crate::Error::InvalidWitness(
                "Nonce must increase".to_string()
            ));
        }
        
        // Validate timestamp
        if witness.private.timestamp < witness.private.prev_timestamp {
            return Err(crate::Error::InvalidWitness(
                "Timestamp must increase".to_string()
            ));
        }
        
        // Validate public inputs are non-zero
        if witness.public.prev_state_hash == "0" {
            return Err(crate::Error::InvalidWitness(
                "Previous state hash cannot be zero".to_string()
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::witness::generator::{TransitionWitness, PublicInputs, PrivateInputs};
    
    #[test]
    fn test_valid_witness() {
        let witness = TransitionWitness {
            public: PublicInputs {
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
            private: PrivateInputs {
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
        
        assert!(WitnessValidator::validate(&witness).is_ok());
    }
}