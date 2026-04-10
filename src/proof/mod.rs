use crate::types::OriginClass;
use crate::utils::*;
use serde::{Deserialize, Serialize};

/// Transition between states
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    pub prev_state_hash: String,
    pub new_state_hash: String,
    pub prev_origin_class: OriginClass,
    pub new_origin_class: OriginClass,
    pub epoch_id: u64,
    pub timestamp: u64,
    pub nonce: u64,
    pub prev_nonce: u64,
}

impl Transition {
    /// Create new transition
    pub fn new(
        prev_state_hash: String,
        new_state_hash: String,
        prev_origin_class: OriginClass,
        new_origin_class: OriginClass,
        epoch_id: u64,
        timestamp: u64,
        nonce: u64,
        prev_nonce: u64,
    ) -> Result<Self, String> {
        // Validate state hashes
        validate_state_hash(&prev_state_hash)?;
        validate_state_hash(&new_state_hash)?;

        // Ensure states are different
        if prev_state_hash == new_state_hash {
            return Err("State must change in transition".to_string());
        }

        // Validate nonce sequence
        if nonce != prev_nonce + 1 {
            return Err("Nonce must increment by 1".to_string());
        }

        Ok(Transition {
            prev_state_hash,
            new_state_hash,
            prev_origin_class,
            new_origin_class,
            epoch_id,
            timestamp,
            nonce,
            prev_nonce,
        })
    }

    /// Get transition hash
    pub fn hash(&self) -> String {
        let combined = format!(
            "{}{}{}{}{}{}{}{}",
            self.prev_state_hash,
            self.new_state_hash,
            self.prev_origin_class as u8,
            self.new_origin_class as u8,
            self.epoch_id,
            self.timestamp,
            self.nonce,
            self.prev_nonce,
        );
        hash_state(combined.as_bytes())
    }
}

/// Lineage commitment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageCommitment {
    pub commitment: String,
    pub depth: u64,
    pub genesis_hash: String,
}

impl LineageCommitment {
    /// Create new lineage commitment
    pub fn new(commitment: String, depth: u64, genesis_hash: String) -> Result<Self, String> {
        validate_state_hash(&commitment)?;
        validate_state_hash(&genesis_hash)?;

        Ok(LineageCommitment {
            commitment,
            depth,
            genesis_hash,
        })
    }

    /// Genesis lineage commitment
    pub fn genesis(genesis_hash: String) -> Result<Self, String> {
        validate_state_hash(&genesis_hash)?;
        Ok(LineageCommitment {
            commitment: genesis_hash.clone(),
            depth: 0,
            genesis_hash,
        })
    }

    /// Update lineage with new transition
    pub fn update(&self, transition_hash: &str) -> Result<Self, String> {
        validate_state_hash(transition_hash)?;

        let combined = format!(
            "{}{}{}",
            self.commitment, transition_hash, self.depth + 1
        );
        let new_commitment = hash_state(combined.as_bytes());

        Ok(LineageCommitment {
            commitment: new_commitment,
            depth: self.depth + 1,
            genesis_hash: self.genesis_hash.clone(),
        })
    }
}

/// Witness for proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Witness {
    pub prev_state_hash: String,
    pub new_state_hash: String,
    pub prev_lineage_commitment: String,
    pub new_lineage_commitment: String,
    pub prev_origin_class: u8,
    pub new_origin_class: u8,
    pub epoch_id: u64,
    pub timestamp: u64,
    pub nonce: u64,
    pub prev_nonce: u64,
    pub prev_counters: Vec<u32>,
    pub rate_limits: Vec<u32>,
    pub policy_proof: Vec<String>,
    pub policy_indices: Vec<u8>,
    pub authorization_valid: u8,
    pub prev_epoch_id: u64,
    pub prev_depth: u64,
    pub expected_genesis_hash: String,
    pub policy_root: String,
    pub prev_counter_commitment: String,
    pub prev_timestamp: u64,
}

impl Witness {
    /// Create witness from transition and lineage
    pub fn from_transition(
        transition: &Transition,
        prev_lineage: &LineageCommitment,
        new_lineage: &LineageCommitment,
        prev_counters: Vec<u32>,
        rate_limits: Vec<u32>,
        policy_root: String,
    ) -> Result<Self, String> {
        // Ensure arrays have correct length
        if prev_counters.len() != 7 {
            return Err(format!("Expected 7 counters, got {}", prev_counters.len()));
        }
        if rate_limits.len() != 7 {
            return Err(format!("Expected 7 rate limits, got {}", rate_limits.len()));
        }

        Ok(Witness {
            prev_state_hash: transition.prev_state_hash.clone(),
            new_state_hash: transition.new_state_hash.clone(),
            prev_lineage_commitment: prev_lineage.commitment.clone(),
            new_lineage_commitment: new_lineage.commitment.clone(),
            prev_origin_class: transition.prev_origin_class as u8,
            new_origin_class: transition.new_origin_class as u8,
            epoch_id: transition.epoch_id,
            timestamp: transition.timestamp,
            nonce: transition.nonce,
            prev_nonce: transition.prev_nonce,
            prev_counters,
            rate_limits,
            policy_proof: vec!["0".to_string(); 6],
            policy_indices: vec![0; 6],
            authorization_valid: 1,
            prev_epoch_id: transition.epoch_id,
            prev_depth: prev_lineage.depth,
            expected_genesis_hash: prev_lineage.genesis_hash.clone(),
            policy_root,
            prev_counter_commitment: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            prev_timestamp: transition.timestamp.saturating_sub(1),
        })
    }

    /// Serialize to JSON for circom input
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Save to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_creation() {
        let prev = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let new = "0x0000000000000000000000000000000000000000000000000000000000000001";

        let transition = Transition::new(
            prev.to_string(),
            new.to_string(),
            OriginClass::Genesis,
            OriginClass::User,
            0,
            1000,
            1,
            0,
        );

        assert!(transition.is_ok());
    }

    #[test]
    fn test_lineage_update() {
        let genesis = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let lineage = LineageCommitment::genesis(genesis.to_string()).unwrap();

        assert_eq!(lineage.depth, 0);

        let transition_hash = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let updated = lineage.update(transition_hash).unwrap();

        assert_eq!(updated.depth, 1);
        assert_ne!(updated.commitment, lineage.commitment);
    }

    #[test]
    fn test_witness_creation() {
        let prev = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let new = "0x0000000000000000000000000000000000000000000000000000000000000001";

        let transition = Transition::new(
            prev.to_string(),
            new.to_string(),
            OriginClass::Genesis,
            OriginClass::User,
            0,
            1000,
            1,
            0,
        )
        .unwrap();

        let prev_lineage = LineageCommitment::genesis(prev.to_string()).unwrap();
        let new_lineage = prev_lineage
            .update(&"0x1111111111111111111111111111111111111111111111111111111111111111")
            .unwrap();

        let witness = Witness::from_transition(
            &transition,
            &prev_lineage,
            &new_lineage,
            vec![0, 0, 0, 0, 0, 0, 0],
            vec![1, 4294967295, 10, 100, 5, 1000, 1],
            "0x000000000000000000000000000000000000000000000000d8e770f2f5a1ff14"
                .to_string(),
        );

        assert!(witness.is_ok());
    }
}