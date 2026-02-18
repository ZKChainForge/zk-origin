//! State transition types

use super::OriginClass;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a state transition in the lineage.
///
/// A transition captures the change from one state to another,
/// along with metadata about who authorized the change.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    /// Hash of the previous state
    pub prev_state_hash: [u8; 32],
    
    /// Hash of the new state
    pub new_state_hash: [u8; 32],
    
    /// Origin class of this transition
    pub origin_class: OriginClass,
    
    /// Timestamp of the transition (Unix seconds)
    pub timestamp: u64,
    
    /// Optional metadata (not included in proofs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TransitionMetadata>,
}

impl Transition {
    /// Create a new transition
    pub fn new(
        prev_state_hash: [u8; 32],
        new_state_hash: [u8; 32],
        origin_class: OriginClass,
        timestamp: u64,
    ) -> Self {
        Self {
            prev_state_hash,
            new_state_hash,
            origin_class,
            timestamp,
            metadata: None,
        }
    }

    /// Create a genesis transition
    pub fn genesis(genesis_state_hash: [u8; 32], timestamp: u64) -> Self {
        Self {
            prev_state_hash: [0u8; 32],
            new_state_hash: genesis_state_hash,
            origin_class: OriginClass::Genesis,
            timestamp,
            metadata: None,
        }
    }

    /// Add metadata to the transition
    pub fn with_metadata(mut self, metadata: TransitionMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Check if this is a genesis transition
    pub fn is_genesis(&self) -> bool {
        self.origin_class == OriginClass::Genesis
    }

    /// Compute the epoch for this transition
    pub fn epoch(&self, epoch_duration: u64) -> u64 {
        self.timestamp / epoch_duration
    }

    /// Compute the transition hash (for lineage commitment)
    pub fn compute_hash(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(&self.prev_state_hash);
        hasher.update(&self.new_state_hash);
        hasher.update(&[self.origin_class as u8]);
        hasher.update(&self.timestamp.to_le_bytes());
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

impl fmt::Debug for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transition")
            .field("prev_state", &hex::encode(&self.prev_state_hash[..4]))
            .field("new_state", &hex::encode(&self.new_state_hash[..4]))
            .field("origin", &self.origin_class)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}...→{}... [{}]",
            hex::encode(&self.prev_state_hash[..4]),
            hex::encode(&self.new_state_hash[..4]),
            self.origin_class
        )
    }
}

/// Optional metadata for a transition (not proven)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionMetadata {
    /// Human-readable description
    pub description: Option<String>,
    
    /// Transaction hash (if applicable)
    pub tx_hash: Option<String>,
    
    /// Block number (if applicable)
    pub block_number: Option<u64>,
    
    /// Additional JSON data
    pub extra: Option<serde_json::Value>,
}

impl TransitionMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self {
            description: None,
            tx_hash: None,
            block_number: None,
            extra: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set transaction hash
    pub fn with_tx_hash(mut self, hash: impl Into<String>) -> Self {
        self.tx_hash = Some(hash.into());
        self
    }
}

impl Default for TransitionMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// A sequence of transitions forming a lineage
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TransitionSequence {
    /// The transitions in order
    pub transitions: Vec<Transition>,
}

impl TransitionSequence {
    /// Create a new empty sequence
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }

    /// Add a transition to the sequence
    pub fn push(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }

    /// Get the number of transitions
    pub fn len(&self) -> usize {
        self.transitions.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty()
    }

    /// Get the latest state hash
    pub fn latest_state_hash(&self) -> Option<[u8; 32]> {
        self.transitions.last().map(|t| t.new_state_hash)
    }

    /// Validate that transitions form a valid chain
    pub fn validate_chain(&self) -> bool {
        if self.transitions.is_empty() {
            return true;
        }

        for window in self.transitions.windows(2) {
            if window[0].new_state_hash != window[1].prev_state_hash {
                return false;
            }
        }
        true
    }

    /// Get origin class sequence
    pub fn origin_sequence(&self) -> Vec<OriginClass> {
        self.transitions.iter().map(|t| t.origin_class).collect()
    }
}

impl IntoIterator for TransitionSequence {
    type Item = Transition;
    type IntoIter = std::vec::IntoIter<Transition>;

    fn into_iter(self) -> Self::IntoIter {
        self.transitions.into_iter()
    }
}

impl<'a> IntoIterator for &'a TransitionSequence {
    type Item = &'a Transition;
    type IntoIter = std::slice::Iter<'a, Transition>;

    fn into_iter(self) -> Self::IntoIter {
        self.transitions.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_creation() {
        let prev = [1u8; 32];
        let new = [2u8; 32];
        let transition = Transition::new(prev, new, OriginClass::User, 1000);
        
        assert_eq!(transition.prev_state_hash, prev);
        assert_eq!(transition.new_state_hash, new);
        assert_eq!(transition.origin_class, OriginClass::User);
        assert_eq!(transition.timestamp, 1000);
    }

    #[test]
    fn test_genesis_transition() {
        let genesis = Transition::genesis([42u8; 32], 0);
        
        assert!(genesis.is_genesis());
        assert_eq!(genesis.prev_state_hash, [0u8; 32]);
        assert_eq!(genesis.origin_class, OriginClass::Genesis);
    }

    #[test]
    fn test_transition_hash_deterministic() {
        let t1 = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 1000);
        let t2 = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 1000);
        
        assert_eq!(t1.compute_hash(), t2.compute_hash());
    }

    #[test]
    fn test_transition_hash_differs() {
        let t1 = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 1000);
        let t2 = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 1000);
        
        assert_ne!(t1.compute_hash(), t2.compute_hash());
    }

    #[test]
    fn test_epoch_calculation() {
        let transition = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 86400 * 3 + 100);
        
        // With 1-day epochs
        assert_eq!(transition.epoch(86400), 3);
    }

    #[test]
    fn test_sequence_validation() {
        let mut seq = TransitionSequence::new();
        
        let t1 = Transition::new([0u8; 32], [1u8; 32], OriginClass::Genesis, 0);
        let t2 = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 100);
        let t3 = Transition::new([2u8; 32], [3u8; 32], OriginClass::User, 200);
        
        seq.push(t1);
        seq.push(t2);
        seq.push(t3);
        
        assert!(seq.validate_chain());
    }

    #[test]
    fn test_sequence_invalid_chain() {
        let mut seq = TransitionSequence::new();
        
        let t1 = Transition::new([0u8; 32], [1u8; 32], OriginClass::Genesis, 0);
        let t2 = Transition::new([99u8; 32], [2u8; 32], OriginClass::User, 100); // Wrong prev
        
        seq.push(t1);
        seq.push(t2);
        
        assert!(!seq.validate_chain());
    }

    #[test]
    fn test_serialization() {
        let transition = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 12345)
            .with_metadata(TransitionMetadata::new().with_description("Test"));
        
        let json = serde_json::to_string(&transition).unwrap();
        let recovered: Transition = serde_json::from_str(&json).unwrap();
        
        assert_eq!(transition.origin_class, recovered.origin_class);
        assert_eq!(transition.timestamp, recovered.timestamp);
    }
}