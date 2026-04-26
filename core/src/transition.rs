// core/src/transition.rs

use crate::{State, OriginPolicy, error::{Error, Result}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A state transition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    /// Previous state
    pub prev_state: State,
    
    /// New state
    pub new_state: State,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Nonce (monotonically increasing)
    pub nonce: u64,
    
    /// Who initiated
    pub initiator: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Transition {
    /// Create a new transition
    pub fn new(
        prev_state: State,
        new_state: State,
        initiator: String,
        nonce: u64,
    ) -> Result<Self> {
        // Validate nonce increases
        if nonce <= prev_state.nonce {
            return Err(Error::InvalidNonce(
                format!("Nonce {} must be greater than {}", nonce, prev_state.nonce)
            ));
        }
        
        // Validate states are different
        if prev_state.hash == new_state.hash {
            return Err(Error::InvalidState(
                "States must be different".to_string()
            ));
        }
        
        // Validate timestamps increase
        if new_state.timestamp < prev_state.timestamp {
            return Err(Error::InvalidTimestamp(
                format!("Time must flow forward: {} < {}", 
                    new_state.timestamp, prev_state.timestamp)
            ));
        }
        
        Ok(Transition {
            prev_state,
            new_state: new_state.clone(),
            timestamp: new_state.timestamp,
            nonce,
            initiator,
            metadata: HashMap::new(),
        })
    }
    
    /// Validate transition against policy
    pub fn is_valid(&self, _policy: &OriginPolicy) -> bool {
        // Check states
        if !self.prev_state.is_valid() || !self.new_state.is_valid() {
            return false;
        }
        
        // Check nonce
        if self.nonce <= self.prev_state.nonce {
            return false;
        }
        
        // Check states are different
        if self.prev_state.hash == self.new_state.hash {
            return false;
        }
        
        // Check time
        if self.new_state.timestamp < self.prev_state.timestamp {
            return false;
        }
        
        true
    }
    
    /// Get transition size
    pub fn size_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StateData;
    
    fn create_test_state(nonce: u64, timestamp: u64) -> State {
        State::new(
            StateData::default(),
            timestamp,
            nonce,
        )
    }
    
    #[test]
    fn test_valid_transition() {
        let prev = create_test_state(0, 1000);
        let new = create_test_state(1, 2000);
        
        let result = Transition::new(prev, new, "user".to_string(), 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_invalid_nonce() {
        let prev = create_test_state(0, 1000);
        let new = create_test_state(1, 2000);
        
        let result = Transition::new(prev, new, "user".to_string(), 0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_same_state() {
        let state = create_test_state(0, 1000);
        let result = Transition::new(state.clone(), state, "user".to_string(), 1);
        assert!(result.is_err());
    }
}