use serde::{Deserialize, Serialize};

/// Lineage commitment representing the complete history of a state
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageCommitment {
    /// The actual commitment value (Poseidon hash)
    pub commitment: [u8; 32],
    /// Current lineage depth
    pub depth: u64,
}

impl LineageCommitment {
    /// Create a new lineage commitment
    pub fn new(commitment: [u8; 32], depth: u64) -> Self {
        Self { commitment, depth }
    }
    
    /// Create genesis lineage commitment
    pub fn genesis(genesis_state_hash: [u8; 32]) -> Self {
        // In production, this would compute Poseidon(state_hash, 0, 0)
        Self {
            commitment: genesis_state_hash, // Placeholder
            depth: 0,
        }
    }
    
    /// Get commitment as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.commitment)
    }
}

impl std::fmt::Display for LineageCommitment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lineage(depth={}, commit=0x{}...)", 
               self.depth, 
               &self.to_hex()[..16])
    }
}