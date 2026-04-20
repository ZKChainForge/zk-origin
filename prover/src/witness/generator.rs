//! Witness Generator: Create circuit inputs from transitions
//!
//! Generates the witness (all circuit inputs) for a state transition,
//! ready for proof generation.

use crate::hash::poseidon::PoseidonHash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Witness for a single transition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransitionWitness {
    /// Public inputs (visible in proof)
    pub public: PublicInputs,
    
    /// Private inputs (hidden in proof)
    pub private: PrivateInputs,
}

/// Public inputs (12 total for Groth16)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicInputs {
    // Outputs (indices 0-2)
    pub new_lineage_commitment: String,
    pub new_counter_commitment: String,
    pub lineage_valid: u32,
    
    // Inputs (indices 3-11)
    pub prev_state_hash: String,
    pub new_state_hash: String,
    pub epoch_id: u32,
    pub prev_origin_class: u8,
    pub new_origin_class: u8,
    pub prev_lineage_commitment: String,
    pub prev_counter_commitment: String,
    pub policy_root: String,
    pub expected_genesis_hash: String,
}

/// Private inputs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateInputs {
    // Epoch & depth
    pub prev_epoch_id: u32,
    pub prev_depth: u32,
    
    // Nonce & timestamp
    pub nonce: u64,
    pub prev_nonce: u64,
    pub timestamp: u64,
    pub prev_timestamp: u64,
    
    // Policy
    pub policy_proof: Vec<String>,
    pub policy_indices: Vec<u8>,
    
    // Counters
    pub prev_counters: Vec<u32>,
    pub rate_limits: Vec<u32>,
    
    // Authorization
    pub public_key_x: Option<String>,
    pub public_key_y: Option<String>,
    pub signature_r: Option<String>,
    pub signature_s: Option<String>,
    pub authorization_valid: u32,
}

/// Witness generator
pub struct WitnessGenerator {
    policy_root: [u8; 32],
    genesis_hash: [u8; 32],
    hasher: PoseidonHash,
}

impl WitnessGenerator {
    pub fn new(policy_root: [u8; 32], genesis_hash: [u8; 32]) -> Self {
        WitnessGenerator {
            policy_root,
            genesis_hash,
            hasher: PoseidonHash::new(),
        }
    }
    
    /// Generate witness for a transition
    pub fn generate(
        &self,
        prev_state_hash: [u8; 32],
        new_state_hash: [u8; 32],
        prev_origin_class: u8,
        new_origin_class: u8,
        prev_lineage_commitment: [u8; 32],
        prev_counter_commitment: [u8; 32],
        prev_counters: Vec<u32>,
        prev_depth: u32,
        epoch_id: u32,
        nonce: u64,
        prev_nonce: u64,
        timestamp: u64,
        prev_timestamp: u64,
        policy_merkle_proof: Vec<[u8; 32]>,
        policy_indices: Vec<u8>,
    ) -> Result<TransitionWitness, String> {
        // Validate inputs
        if nonce <= prev_nonce {
            return Err("Nonce must increase".to_string());
        }
        
        if timestamp < prev_timestamp {
            return Err("Timestamp must increase".to_string());
        }
        
        if prev_state_hash == new_state_hash {
            return Err("States must be different".to_string());
        }
        
        // Compute new counter commitment
        let new_counters = self.compute_new_counters(
            epoch_id,
            new_origin_class as usize,
            &prev_counters,
        );
        
        let new_counter_commitment = self.compute_counter_commitment(
            epoch_id,
            &new_counters,
        );
        
        // Compute transition hash
        let transition_hash = self.compute_transition_hash(
            prev_state_hash,
            new_state_hash,
            new_origin_class,
            timestamp,
            nonce,
        );
        
        // Compute new lineage commitment
        let new_lineage_commitment = self.compute_lineage_commitment(
            prev_lineage_commitment,
            transition_hash,
            prev_depth + 1,
        );
        
        // Create public inputs
        let public = PublicInputs {
            new_lineage_commitment: format!("{}", field_element_to_string(new_lineage_commitment)),
            new_counter_commitment: format!("{}", field_element_to_string(new_counter_commitment)),
            lineage_valid: 1,
            prev_state_hash: format!("{}", field_element_to_string(prev_state_hash)),
            new_state_hash: format!("{}", field_element_to_string(new_state_hash)),
            epoch_id,
            prev_origin_class,
            new_origin_class,
            prev_lineage_commitment: format!("{}", field_element_to_string(prev_lineage_commitment)),
            prev_counter_commitment: format!("{}", field_element_to_string(prev_counter_commitment)),
            policy_root: format!("{}", field_element_to_string(self.policy_root)),
            expected_genesis_hash: format!("{}", field_element_to_string(self.genesis_hash)),
        };
        
        // Create private inputs
        let private = PrivateInputs {
            prev_epoch_id: epoch_id,
            prev_depth,
            nonce,
            prev_nonce,
            timestamp,
            prev_timestamp,
            policy_proof: policy_merkle_proof
                .iter()
                .map(|h| format!("{}", field_element_to_string(*h)))
                .collect(),
            policy_indices,
            prev_counters,
            rate_limits: vec![1, u32::MAX, 10, 100, 5, 1000, 1],
            public_key_x: None,
            public_key_y: None,
            signature_r: None,
            signature_s: None,
            authorization_valid: 1,
        };
        
        Ok(TransitionWitness { public, private })
    }
    
    fn compute_new_counters(
        &self,
        epoch_id: u32,
        origin_class: usize,
        prev_counters: &[u32],
    ) -> Vec<u32> {
        let mut new_counters = prev_counters.to_vec();
        new_counters[origin_class] = prev_counters[origin_class].saturating_add(1);
        new_counters
    }
    
    fn compute_counter_commitment(
        &self,
        epoch_id: u32,
        counters: &[u32],
    ) -> [u8; 32] {
        // Hash: Poseidon(epoch, counter[0], counter[1], ..., counter[6])
        let mut data = vec![epoch_id as u8];
        for counter in counters {
            data.extend_from_slice(&counter.to_le_bytes());
        }
        
        let mut hash = [0u8; 32];
        let result = self.hasher.hash(&data);
        hash.copy_from_slice(&result[..32]);
        hash
    }
    
    fn compute_transition_hash(
        &self,
        prev_state: [u8; 32],
        new_state: [u8; 32],
        origin_class: u8,
        timestamp: u64,
        nonce: u64,
    ) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&prev_state);
        data.extend_from_slice(&new_state);
        data.push(origin_class);
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());
        
        let mut hash = [0u8; 32];
        let result = self.hasher.hash(&data);
        hash.copy_from_slice(&result[..32]);
        hash
    }
    
    fn compute_lineage_commitment(
        &self,
        prev_lineage: [u8; 32],
        transition_hash: [u8; 32],
        depth: u32,
    ) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&prev_lineage);
        data.extend_from_slice(&transition_hash);
        data.extend_from_slice(&depth.to_le_bytes());
        
        let mut hash = [0u8; 32];
        let result = self.hasher.hash(&data);
        hash.copy_from_slice(&result[..32]);
        hash
    }
}

fn field_element_to_string(hash: [u8; 32]) -> String {
    // Convert hash to decimal string (Bn254 field element)
    use num_bigint::BigInt;
    use std::str::FromStr;
    
    let mut bytes = hash.to_vec();
    bytes.reverse();
    let big_int = BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes);
    big_int.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_witness_generation() {
        let generator = WitnessGenerator::new([0u8; 32], [0u8; 32]);
        
        let result = generator.generate(
            [1u8; 32],
            [2u8; 32],
            1,
            1,
            [0u8; 32],
            [0u8; 32],
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
        
        assert!(result.is_ok());
    }
}