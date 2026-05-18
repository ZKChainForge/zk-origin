use crate::{error::Result, hash::Hash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State data structure
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StateData {
    pub accounts: HashMap<String, AccountState>,
    pub balances: HashMap<String, u128>,
    pub metadata: HashMap<String, String>,
}

impl StateData {
    /// Validate state data consistency
    pub fn validate(&self) -> Result<()> {
        // Check no negative balances (stored as u128)
        if self.balances.values().any(|&b| b > i128::MAX as u128) {
            return Err(crate::error::Error::invalid_state("Balance overflow"));
        }

        Ok(())
    }
}

/// Account state
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AccountState {
    pub nonce: u64,
    pub balance: u128,
    pub code_hash: Hash,
}

/// Production-grade state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub id: Vec<u8>,
    pub data: StateData,
    pub hash: Hash,
    pub timestamp: u64,
    pub nonce: u64,
}

impl State {
    /// Create genesis state with validation
    pub fn genesis(data: StateData) -> Result<Self> {
        data.validate()?;

        let serialized = serde_json::to_vec(&data)?;
        let hash = crate::hash::keccak256(&serialized);

        Ok(State {
            id: hash.as_slice().to_vec(),
            data,
            hash,
            timestamp: 0,
            nonce: 0,
        })
    }

    /// Create new state with validation
    pub fn new(data: StateData, timestamp: u64, nonce: u64) -> Result<Self> {
        data.validate()?;

        if nonce == u64::MAX {
            return Err(crate::error::Error::invalid_nonce("Nonce overflow"));
        }

        let serialized = serde_json::to_vec(&data)?;
        let hash = crate::hash::keccak256(&serialized);

        Ok(State {
            id: hash.as_slice().to_vec(),
            data,
            hash,
            timestamp,
            nonce,
        })
    }

    /// Validate state consistency
    pub fn validate(&self) -> Result<()> {
        self.data.validate()?;

        // Verify hash matches data
        let serialized = serde_json::to_vec(&self.data)?;
        let expected_hash = crate::hash::keccak256(&serialized);

        if self.hash != expected_hash {
            return Err(crate::error::Error::StateHashMismatch {
                expected: expected_hash.to_hex(),
                actual: self.hash.to_hex(),
            });
        }

        // Verify nonce not overflowed
        if self.nonce == u64::MAX {
            return Err(crate::error::Error::invalid_nonce("Nonce at maximum"));
        }

        Ok(())
    }
}

/// Lineage representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lineage {
    pub depth: u32,
    pub genesis_hash: Hash,
    pub lineage_commitment: Hash,
    pub transitions: Vec<(State, State)>,
}

/// Production state machine
pub struct StateMachine {
    current_state: State,
    history: Vec<crate::Transition>,
    policy: crate::OriginPolicy,
}

impl StateMachine {
    /// Create new machine with genesis
    pub fn new(genesis: State, policy: crate::OriginPolicy) -> Result<Self> {
        genesis.validate()?;

        Ok(StateMachine {
            current_state: genesis,
            history: Vec::new(),
            policy,
        })
    }

    /// Apply transition
    pub fn apply_transition(&mut self, transition: crate::Transition) -> Result<()> {
        // Validate transition
        transition.validate()?;

        // Verify previous state matches
        if transition.prev_state.hash != self.current_state.hash {
            return Err(crate::error::Error::InvalidState {
                context: format!(
                    "Previous state mismatch: expected {}, got {}",
                    self.current_state.hash, transition.prev_state.hash
                ),
            });
        }

        // Check policy
        if !self
            .policy
            .is_allowed(transition.prev_origin, transition.new_origin)
        {
            return Err(crate::error::Error::policy_violation(format!(
                "Transition from {} to {} not allowed",
                transition.prev_origin, transition.new_origin
            )));
        }

        // Record and update
        self.history.push(transition.clone());
        self.current_state = transition.new_state.clone();

        Ok(())
    }

    /// Get current state
    pub fn get_current_state(&self) -> &State {
        &self.current_state
    }

    /// Get history
    pub fn get_history(&self) -> &[crate::Transition] {
        &self.history
    }

    /// Get lineage
    pub fn get_lineage(&self) -> Result<Lineage> {
        let genesis = State::genesis(StateData::default())?;

        let mut lineage_hash = genesis.hash;

        for transition in &self.history {
            lineage_hash = crate::hash::hash_lineage(
                lineage_hash,
                crate::hash::hash_transition(
                    transition.prev_state.hash,
                    transition.new_state.hash,
                    transition.prev_origin.as_u8(),
                    transition.new_state.timestamp,
                    transition.new_state.nonce,
                ),
                self.history.len() as u32,
            );
        }

        Ok(Lineage {
            depth: self.history.len() as u32,
            genesis_hash: genesis.hash,
            lineage_commitment: lineage_hash,
            transitions: self
                .history
                .iter()
                .map(|t| (t.prev_state.clone(), t.new_state.clone()))
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_state_creation() {
        let state = State::new(StateData::default(), 1000, 0);
        assert!(state.is_ok());
    }

    #[test]
    fn test_state_validation() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        let state = State::new(StateData::default(), 1000, nonce).unwrap();
        assert!(state.validate().is_ok());
    }

    #[test]
    fn test_genesis_creation() {
        let genesis = State::genesis(StateData::default());
        assert!(genesis.is_ok());
        let state = genesis.unwrap();
        assert_eq!(state.nonce, 0);
        assert_eq!(state.timestamp, 0);
    }
}
