use super::Origin;
use serde::{Deserialize, Serialize};

/// A state transition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    /// Hash of the previous state
    pub prev_state: [u8; 32],
    /// Hash of the new state
    pub new_state: [u8; 32],
    /// Origin class of this transition
    pub origin: Origin,
    /// Timestamp of the transition
    pub timestamp: u64,
}

impl Transition {
    /// Create a new transition
    pub fn new(
        prev_state: [u8; 32],
        new_state: [u8; 32],
        origin: Origin,
        timestamp: u64,
    ) -> Self {
        Self {
            prev_state,
            new_state,
            origin,
            timestamp,
        }
    }
    
    /// Compute the transition hash
    pub fn hash(&self) -> [u8; 32] {
        // In production, compute Poseidon(prev, new, origin, timestamp)
        [0u8; 32] // Placeholder
    }
}