// core/src/lib.rs

#![warn(missing_docs)]

pub mod state;
pub mod origin;
pub mod policy;
pub mod error;
pub mod hash;
pub mod utils;

// Transition is in state module, re-export it
pub use state::machine::{State, StateData, StateMachine, Lineage};
pub use origin::{OriginClass, OriginDetector, OriginContext};
pub use policy::OriginPolicy;
pub use error::{Error, Result};
pub use hash::{keccak256, hash_state, hash_transition};

/// Transition struct - define here or import
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
    
    /// Nonce
    pub nonce: u64,
    
    /// Initiator
    pub initiator: String,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl Transition {
    /// Create new transition
    pub fn new(
        prev_state: State,
        new_state: State,
        initiator: String,
        nonce: u64,
    ) -> Result<Self> {
        // Validate nonce
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
        
        // Validate timestamps
        if new_state.timestamp < prev_state.timestamp {
            return Err(Error::InvalidTimestamp(
                "Time must flow forward".to_string()
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
    
    /// Validate transition
    pub fn is_valid(&self, _policy: &OriginPolicy) -> bool {
        if !self.prev_state.is_valid() || !self.new_state.is_valid() {
            return false;
        }
        
        if self.nonce <= self.prev_state.nonce {
            return false;
        }
        
        if self.prev_state.hash == self.new_state.hash {
            return false;
        }
        
        if self.new_state.timestamp < self.prev_state.timestamp {
            return false;
        }
        
        true
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}