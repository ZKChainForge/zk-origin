//! Witness generation for state transitions

use crate::error::{ProverError, Result};
use crate::hash::{sha3_256, Hash};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

/// Public witness inputs (visible in proof)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicWitness {
    /// New lineage commitment
    pub new_lineage_commitment: String,
    /// New counter commitment
    pub new_counter_commitment: String,
    /// Lineage validity flag
    pub lineage_valid: u32,
    /// Previous state hash
    pub prev_state_hash: String,
    /// New state hash
    pub new_state_hash: String,
    /// Epoch ID
    pub epoch_id: u32,
    /// Previous origin class
    pub prev_origin_class: u8,
    /// New origin class
    pub new_origin_class: u8,
    /// Previous lineage commitment
    pub prev_lineage_commitment: String,
    /// Previous counter commitment
    pub prev_counter_commitment: String,
    /// Policy root hash
    pub policy_root: String,
    /// Expected genesis hash
    pub expected_genesis_hash: String,
}

/// Private witness inputs (hidden in proof)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateWitness {
    /// Previous epoch ID
    pub prev_epoch_id: u32,
    /// Previous depth
    pub prev_depth: u32,
    /// Nonce
    pub nonce: u64,
    /// Previous nonce
    pub prev_nonce: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Previous timestamp
    pub prev_timestamp: u64,
    /// Policy merkle proof
    pub policy_proof: Vec<String>,
    /// Policy indices
    pub policy_indices: Vec<u8>,
    /// Previous counters
    pub prev_counters: Vec<u32>,
    /// Rate limits
    pub rate_limits: Vec<u32>,
    /// Public key X coordinate
    pub public_key_x: Option<String>,
    /// Public key Y coordinate
    pub public_key_y: Option<String>,
    /// Signature R component
    pub signature_r: Option<String>,
    /// Signature S component
    pub signature_s: Option<String>,
    /// Authorization validity flag
    pub authorization_valid: u32,
}

/// Complete transition witness
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransitionWitness {
    /// Public witness data
    pub public: PublicWitness,
    /// Private witness data
    pub private: PrivateWitness,
}

impl TransitionWitness {
    /// Validate witness integrity
    pub fn validate(&self) -> Result<()> {
        // Validate nonce
        if self.private.nonce <= self.private.prev_nonce {
            return Err(ProverError::invalid_nonce(format!(
                "Nonce must increase: {} > {}",
                self.private.nonce, self.private.prev_nonce
            )));
        }

        // Validate timestamp
        if self.private.timestamp < self.private.prev_timestamp {
            return Err(ProverError::invalid_timestamp(format!(
                "Timestamp must increase: {} >= {}",
                self.private.timestamp, self.private.prev_timestamp
            )));
        }

        // Validate state hashes are not equal
        if self.public.prev_state_hash == self.public.new_state_hash {
            return Err(ProverError::invalid_state("State must change"));
        }

        // Validate state hashes are non-zero
        if self.public.prev_state_hash == "0" || self.public.new_state_hash == "0" {
            return Err(ProverError::invalid_state("State hash cannot be zero"));
        }

        // Validate origin classes
        if self.public.prev_origin_class >= 7 || self.public.new_origin_class >= 7 {
            return Err(ProverError::invalid_state("Invalid origin class"));
        }

        // Validate counter arrays
        if self.private.prev_counters.len() != 7 {
            return Err(ProverError::invalid_witness(format!(
                "Expected 7 counters, got {}",
                self.private.prev_counters.len()
            )));
        }

        if self.private.rate_limits.len() != 7 {
            return Err(ProverError::invalid_witness(format!(
                "Expected 7 rate limits, got {}",
                self.private.rate_limits.len()
            )));
        }

        // Validate authorization
        if self.private.authorization_valid != 0 && self.private.authorization_valid != 1 {
            return Err(ProverError::authorization_failed(
                "authorization_valid must be 0 or 1",
            ));
        }

        Ok(())
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "public": {
                "newLineageCommitment": self.public.new_lineage_commitment,
                "newCounterCommitment": self.public.new_counter_commitment,
                "lineageValid": self.public.lineage_valid,
                "prevStateHash": self.public.prev_state_hash,
                "newStateHash": self.public.new_state_hash,
                "epochId": self.public.epoch_id,
                "prevOriginClass": self.public.prev_origin_class,
                "newOriginClass": self.public.new_origin_class,
                "prevLineageCommitment": self.public.prev_lineage_commitment,
                "prevCounterCommitment": self.public.prev_counter_commitment,
                "policyRoot": self.public.policy_root,
                "expectedGenesisHash": self.public.expected_genesis_hash,
            },
            "private": {
                "prevEpochId": self.private.prev_epoch_id,
                "prevDepth": self.private.prev_depth,
                "nonce": self.private.nonce,
                "prevNonce": self.private.prev_nonce,
                "timestamp": self.private.timestamp,
                "prevTimestamp": self.private.prev_timestamp,
                "policyProof": self.private.policy_proof,
                "policyIndices": self.private.policy_indices,
                "prevCounters": self.private.prev_counters,
                "rateLimits": self.private.rate_limits,
                "publicKeyX": self.private.public_key_x.clone().unwrap_or_else(|| "0".to_string()),
                "publicKeyY": self.private.public_key_y.clone().unwrap_or_else(|| "0".to_string()),
                "signatureR": self.private.signature_r.clone().unwrap_or_else(|| "0".to_string()),
                "signatureS": self.private.signature_s.clone().unwrap_or_else(|| "0".to_string()),
                "authorizationValid": self.private.authorization_valid,
            }
        }))
    }
}

/// Production witness generator
pub struct WitnessGenerator {
    policy_root: Hash,
    genesis_hash: Hash,
}

impl WitnessGenerator {
    /// Create new generator
    pub fn new(policy_root: Hash, genesis_hash: Hash) -> Self {
        WitnessGenerator {
            policy_root,
            genesis_hash,
        }
    }

    /// Generate witness for transition
    #[allow(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        prev_state_hash: Hash,
        new_state_hash: Hash,
        prev_origin_class: u8,
        new_origin_class: u8,
        prev_lineage_commitment: Hash,
        prev_counter_commitment: Hash,
        prev_counters: Vec<u32>,
        prev_depth: u32,
        epoch_id: u32,
        nonce: u64,
        prev_nonce: u64,
        timestamp: u64,
        prev_timestamp: u64,
        policy_merkle_proof: Vec<Hash>,
        policy_indices: Vec<u8>,
    ) -> Result<TransitionWitness> {
        // Validate inputs
        self.validate_inputs(
            prev_state_hash,
            new_state_hash,
            prev_origin_class,
            new_origin_class,
            prev_counters.len(),
            nonce,
            prev_nonce,
            timestamp,
            prev_timestamp,
        )?;

        // Compute new counters
        let new_counters =
            self.compute_new_counters(epoch_id, new_origin_class as usize, &prev_counters)?;

        // Compute commitments
        let new_counter_commitment = self.compute_counter_commitment(epoch_id, &new_counters)?;
        let transition_hash = self.compute_transition_hash(
            prev_state_hash,
            new_state_hash,
            new_origin_class,
            timestamp,
            nonce,
        )?;
        let new_lineage_commitment = self.compute_lineage_commitment(
            prev_lineage_commitment,
            transition_hash,
            prev_depth + 1,
        )?;

        // Create witness
        let witness = TransitionWitness {
            public: PublicWitness {
                new_lineage_commitment: hash_to_field_string(new_lineage_commitment),
                new_counter_commitment: hash_to_field_string(new_counter_commitment),
                lineage_valid: 1,
                prev_state_hash: hash_to_field_string(prev_state_hash),
                new_state_hash: hash_to_field_string(new_state_hash),
                epoch_id,
                prev_origin_class,
                new_origin_class,
                prev_lineage_commitment: hash_to_field_string(prev_lineage_commitment),
                prev_counter_commitment: hash_to_field_string(prev_counter_commitment),
                policy_root: hash_to_field_string(self.policy_root),
                expected_genesis_hash: hash_to_field_string(self.genesis_hash),
            },
            private: PrivateWitness {
                prev_epoch_id: epoch_id,
                prev_depth,
                nonce,
                prev_nonce,
                timestamp,
                prev_timestamp,
                policy_proof: policy_merkle_proof
                    .iter()
                    .map(|h: &Hash| h.to_hex())
                    .collect(),
                policy_indices,
                prev_counters,
                rate_limits: vec![1, u32::MAX, 10, 100, 5, 1000, 1],
                public_key_x: None,
                public_key_y: None,
                signature_r: None,
                signature_s: None,
                authorization_valid: 1,
            },
        };

        // Validate before returning
        witness.validate()?;

        Ok(witness)
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_inputs(
        &self,
        prev_state_hash: Hash,
        new_state_hash: Hash,
        prev_origin_class: u8,
        new_origin_class: u8,
        counter_count: usize,
        nonce: u64,
        prev_nonce: u64,
        timestamp: u64,
        prev_timestamp: u64,
    ) -> Result<()> {
        if prev_origin_class >= 7 {
            return Err(ProverError::invalid_state("Invalid previous origin class"));
        }
        if new_origin_class >= 7 {
            return Err(ProverError::invalid_state("Invalid new origin class"));
        }
        if prev_state_hash == new_state_hash {
            return Err(ProverError::invalid_state("States must be different"));
        }
        if nonce <= prev_nonce {
            return Err(ProverError::invalid_nonce(format!(
                "Nonce must increase: {} > {}",
                nonce, prev_nonce
            )));
        }
        if timestamp < prev_timestamp {
            return Err(ProverError::invalid_timestamp(format!(
                "Timestamp must increase: {} >= {}",
                timestamp, prev_timestamp
            )));
        }
        if counter_count != 7 {
            return Err(ProverError::invalid_witness(format!(
                "Expected 7 counters, got {}",
                counter_count
            )));
        }
        Ok(())
    }

    fn compute_new_counters(
        &self,
        _epoch_id: u32,
        origin_class: usize,
        prev_counters: &[u32],
    ) -> Result<Vec<u32>> {
        if origin_class >= 7 {
            return Err(ProverError::invalid_state("Invalid origin class index"));
        }

        let mut new_counters = prev_counters.to_vec();
        new_counters[origin_class] =
            prev_counters[origin_class].checked_add(1).ok_or_else(|| {
                ProverError::rate_limit_exceeded(format!(
                    "Counter overflow for class {}",
                    origin_class
                ))
            })?;

        Ok(new_counters)
    }

    fn compute_counter_commitment(&self, epoch_id: u32, counters: &[u32]) -> Result<Hash> {
        let mut data = epoch_id.to_le_bytes().to_vec();
        for counter in counters {
            data.extend_from_slice(&counter.to_le_bytes());
        }
        Ok(sha3_256(&data))
    }

    fn compute_transition_hash(
        &self,
        prev_state: Hash,
        new_state: Hash,
        origin_class: u8,
        timestamp: u64,
        nonce: u64,
    ) -> Result<Hash> {
        let mut data = Vec::new();
        data.extend_from_slice(prev_state.as_slice());
        data.extend_from_slice(new_state.as_slice());
        data.push(origin_class);
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());
        Ok(sha3_256(&data))
    }

    fn compute_lineage_commitment(
        &self,
        prev_lineage: Hash,
        transition_hash: Hash,
        depth: u32,
    ) -> Result<Hash> {
        let mut data = Vec::new();
        data.extend_from_slice(prev_lineage.as_slice());
        data.extend_from_slice(transition_hash.as_slice());
        data.extend_from_slice(&depth.to_le_bytes());
        Ok(sha3_256(&data))
    }
}

/// Convert hash to BN254 field element string
fn hash_to_field_string(hash: Hash) -> String {
    let bytes = hash.as_slice().to_vec();
    let big_uint = BigUint::from_bytes_be(&bytes);
    big_uint.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_generation() {
        let generator = WitnessGenerator::new(Hash::default(), Hash::default());

        let witness = generator.generate(
            Hash::from_array([1u8; 32]),
            Hash::from_array([2u8; 32]),
            0,
            1,
            Hash::default(),
            Hash::default(),
            vec![0, 0, 0, 0, 0, 0, 0],
            0,
            0,
            1,
            0,
            1000,
            999,
            vec![],
            vec![],
        );

        assert!(witness.is_ok());
    }

    #[test]
    fn test_witness_validation() {
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

        assert!(witness.validate().is_ok());
    }

    #[test]
    fn test_invalid_nonce() {
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
                nonce: 0,
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

        assert!(witness.validate().is_err());
    }
}