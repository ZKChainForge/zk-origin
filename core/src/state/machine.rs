// core/src/state/machine.rs

use crate::{OriginPolicy, error::Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State data
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StateData {
    /// Account states
    pub accounts: HashMap<String, AccountState>,
    
    /// Balances
    pub balances: HashMap<String, u128>,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Account state
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AccountState {
    /// Nonce
    pub nonce: u64,
    
    /// Balance
    pub balance: u128,
    
    /// Code hash
    pub code_hash: [u8; 32],
}

/// Represents a blockchain state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    /// Unique identifier for this state
    pub id: Vec<u8>,
    
    /// State data (application-specific)
    pub data: StateData,
    
    /// Hash of this state (Keccak256)
    pub hash: [u8; 32],
    
    /// Timestamp when state was created
    pub timestamp: u64,
    
    /// Nonce (monotonically increasing)
    pub nonce: u64,
}

impl State {
    /// Create a new genesis state
    pub fn genesis(data: StateData) -> Self {
        let serialized = serde_json::to_vec(&data).expect("Failed to serialize state");
        let hash = crate::hash::keccak256(&serialized);
        
        State {
            id: hash.to_vec(),
            data,
            hash,
            timestamp: 0,
            nonce: 0,
        }
    }
    
    /// Create a new state
    pub fn new(
        data: StateData,
        timestamp: u64,
        nonce: u64,
    ) -> Self {
        let serialized = serde_json::to_vec(&data).expect("Failed to serialize state");
        let hash = crate::hash::keccak256(&serialized);
        
        State {
            id: hash.to_vec(),
            data,
            hash,
            timestamp,
            nonce,
        }
    }
    
    /// Get state hash
    pub fn get_hash(&self) -> [u8; 32] {
        self.hash
    }
    
    /// Verify state is valid
    pub fn is_valid(&self) -> bool {
        let serialized = serde_json::to_vec(&self.data).expect("Failed to serialize");
        let expected_hash = crate::hash::keccak256(&serialized);
        
        self.hash == expected_hash && self.nonce < u64::MAX
    }
}

/// Lineage: full ancestry of a state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lineage {
    /// Lineage depth
    pub depth: u32,
    
    /// Genesis state hash
    pub genesis_hash: [u8; 32],
    
    /// All transitions
    pub transitions: Vec<(State, State)>,
}

/// State machine: manages state transitions
pub struct StateMachine {
    /// Current state
    current_state: State,
    
    /// History of transitions
    history: Vec<crate::Transition>,
    
    /// Policy for validating transitions
    policy: OriginPolicy,
}

impl StateMachine {
    /// Create new state machine with genesis state
    pub fn new(genesis: State, policy: OriginPolicy) -> Self {
        StateMachine {
            current_state: genesis,
            history: Vec::new(),
            policy,
        }
    }
    
    /// Apply a transition
    pub fn apply_transition(&mut self, transition: crate::Transition) -> Result<()> {
        // Validate transition
        if !transition.is_valid(&self.policy) {
            return Err(crate::error::Error::InvalidTransition(
                "Transition validation failed".to_string()
            ));
        }
        
        // Verify previous state matches current
        if transition.prev_state.hash != self.current_state.hash {
            return Err(crate::error::Error::InvalidState(
                "Previous state mismatch".to_string()
            ));
        }
        
        // Record transition
        self.history.push(transition.clone());
        self.current_state = transition.new_state.clone();
        
        Ok(())
    }
    
    /// Get current state
    pub fn get_current_state(&self) -> &State {
        &self.current_state
    }
    
    /// Get transition history
    pub fn get_history(&self) -> &[crate::Transition] {
        &self.history
    }
    
    /// Get lineage
    pub fn get_lineage(&self) -> Lineage {
        let mut transitions = Vec::new();
        
        // Add genesis
        let genesis = State::genesis(StateData::default());
        
        // Add all transitions
        for transition in &self.history {
            transitions.push((transition.prev_state.clone(), transition.new_state.clone()));
        }
        
        Lineage {
            depth: self.history.len() as u32,
            genesis_hash: genesis.hash,
            transitions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_state(nonce: u64, timestamp: u64) -> State {
        State::new(StateData::default(), timestamp, nonce)
    }
    
    #[test]
    fn test_state_creation() {
        let state = create_test_state(0, 1000);
        assert!(state.is_valid());
        assert_eq!(state.nonce, 0);
    }
    
    #[test]
    fn test_genesis_creation() {
        let genesis = State::genesis(StateData::default());
        assert!(genesis.is_valid());
        assert_eq!(genesis.nonce, 0);
        assert_eq!(genesis.timestamp, 0);
    }
}