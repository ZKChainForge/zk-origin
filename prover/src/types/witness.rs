
//! Step witness types for circuit proving

use super::{OriginClass, Transition};
use serde::{Deserialize, Serialize};

/// Witness data for a single step of the lineage circuit.
///
/// This contains all the private inputs needed to prove
/// a valid state transition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepWitness {
    // Previous state info
    /// Hash of the previous state
    pub prev_state_hash: [u8; 32],
    
    /// Previous lineage commitment
    pub prev_lineage_commitment: [u8; 32],
    
    /// Previous origin class
    pub prev_origin: OriginClass,
    
    /// Previous lineage depth
    pub prev_depth: u64,
    
    // New transition info
    /// Hash of the new state
    pub new_state_hash: [u8; 32],
    
    /// Origin class of this transition
    pub new_origin: OriginClass,
    
    /// Transition timestamp
    pub timestamp: u64,
    
    // Policy proof
    /// Merkle proof path for policy verification
    pub policy_proof: Vec<[u8; 32]>,
    
    /// Path indices (0 = left, 1 = right)
    pub policy_indices: Vec<bool>,
    
    /// Policy tree root
    pub policy_root: [u8; 32],
    
    // Counter info
    /// Current epoch ID
    pub epoch_id: u64,
    
    /// Previous counter values
    pub prev_counters: [u32; 6],
    
    /// Rate limits for each origin class
    pub rate_limits: [u32; 6],
    
    /// Previous counter commitment
    pub prev_counter_commitment: [u8; 32],
}

impl StepWitness {
    /// Create a new witness for a transition
    pub fn new(
        transition: &Transition,
        prev_lineage_commitment: [u8; 32],
        prev_origin: OriginClass,
        prev_depth: u64,
        policy_proof: Vec<[u8; 32]>,
        policy_indices: Vec<bool>,
        policy_root: [u8; 32],
        epoch_id: u64,
        prev_counters: [u32; 6],
        rate_limits: [u32; 6],
        prev_counter_commitment: [u8; 32],
    ) -> Self {
        Self {
            prev_state_hash: transition.prev_state_hash,
            prev_lineage_commitment,
            prev_origin,
            prev_depth,
            new_state_hash: transition.new_state_hash,
            new_origin: transition.origin_class,
            timestamp: transition.timestamp,
            policy_proof,
            policy_indices,
            policy_root,
            epoch_id,
            prev_counters,
            rate_limits,
            prev_counter_commitment,
        }
    }

    /// Create a genesis witness
    pub fn genesis(
        genesis_state_hash: [u8; 32],
        timestamp: u64,
        policy_root: [u8; 32],
        policy_proof: Vec<[u8; 32]>,
        policy_indices: Vec<bool>,
    ) -> Self {
        Self {
            prev_state_hash: [0u8; 32],
            prev_lineage_commitment: [0u8; 32],
            prev_origin: OriginClass::Genesis,
            prev_depth: 0,
            new_state_hash: genesis_state_hash,
            new_origin: OriginClass::Genesis,
            timestamp,
            policy_proof,
            policy_indices,
            policy_root,
            epoch_id: 0,
            prev_counters: [0; 6],
            rate_limits: [1, u32::MAX, 10, 100, 5, 1000],
            prev_counter_commitment: [0u8; 32],
        }
    }

    /// Validate the witness structure (not cryptographic validity)
    pub fn validate_structure(&self) -> Result<(), &'static str> {
        // Check policy proof depth matches indices
        if self.policy_proof.len() != self.policy_indices.len() {
            return Err("Policy proof length mismatch");
        }
        
        // Check origin classes are valid
        if self.prev_origin as u8 > 5 || self.new_origin as u8 > 5 {
            return Err("Invalid origin class");
        }
        
        Ok(())
    }

    /// Compute the transition hash
    pub fn compute_transition_hash(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(&self.prev_state_hash);
        hasher.update(&self.new_state_hash);
        hasher.update(&[self.new_origin as u8]);
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.epoch_id.to_le_bytes());
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Compute the new lineage commitment
    pub fn compute_new_lineage_commitment(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        let transition_hash = self.compute_transition_hash();
        let new_depth = self.prev_depth + 1;
        
        let mut hasher = Sha256::new();
        hasher.update(&self.prev_lineage_commitment);
        hasher.update(&transition_hash);
        hasher.update(&new_depth.to_le_bytes());
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Compute new counter values
    pub fn compute_new_counters(&self) -> [u32; 6] {
        let mut new_counters = self.prev_counters;
        let idx = self.new_origin as usize;
        if idx < new_counters.len() {
            new_counters[idx] = new_counters[idx].saturating_add(1);
        }
        new_counters
    }

    /// Compute new counter commitment
    pub fn compute_new_counter_commitment(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        let new_counters = self.compute_new_counters();
        
        let mut hasher = Sha256::new();
        hasher.update(&self.epoch_id.to_le_bytes());
        for counter in &new_counters {
            hasher.update(&counter.to_le_bytes());
        }
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Get the new depth after this transition
    pub fn new_depth(&self) -> u64 {
        self.prev_depth + 1
    }

    /// Check if this is a genesis witness
    pub fn is_genesis(&self) -> bool {
        self.prev_depth == 0 && self.prev_origin == OriginClass::Genesis
    }
}

impl Default for StepWitness {
    fn default() -> Self {
        Self {
            prev_state_hash: [0u8; 32],
            prev_lineage_commitment: [0u8; 32],
            prev_origin: OriginClass::Genesis,
            prev_depth: 0,
            new_state_hash: [0u8; 32],
            new_origin: OriginClass::User,
            timestamp: 0,
            policy_proof: Vec::new(),
            policy_indices: Vec::new(),
            policy_root: [0u8; 32],
            epoch_id: 0,
            prev_counters: [0; 6],
            rate_limits: [1, u32::MAX, 10, 100, 5, 1000],
            prev_counter_commitment: [0u8; 32],
        }
    }
}

/// Batch of witnesses for multiple steps
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WitnessBatch {
    /// The witnesses in order
    pub witnesses: Vec<StepWitness>,
}

impl WitnessBatch {
    /// Create a new empty batch
    pub fn new() -> Self {
        Self {
            witnesses: Vec::new(),
        }
    }

    /// Add a witness
    pub fn push(&mut self, witness: StepWitness) {
        self.witnesses.push(witness);
    }

    /// Get the number of witnesses
    pub fn len(&self) -> usize {
        self.witnesses.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.witnesses.is_empty()
    }

    /// Get witness at index
    pub fn get(&self, index: usize) -> Option<&StepWitness> {
        self.witnesses.get(index)
    }

    /// Iterate over witnesses
    pub fn iter(&self) -> impl Iterator<Item = &StepWitness> {
        self.witnesses.iter()
    }

    /// Validate all witnesses in the batch
    pub fn validate_all(&self) -> Result<(), &'static str> {
        for witness in &self.witnesses {
            witness.validate_structure()?;
        }
        Ok(())
    }
}

impl IntoIterator for WitnessBatch {
    type Item = StepWitness;
    type IntoIter = std::vec::IntoIter<StepWitness>;

    fn into_iter(self) -> Self::IntoIter {
        self.witnesses.into_iter()
    }
}

impl<'a> IntoIterator for &'a WitnessBatch {
    type Item = &'a StepWitness;
    type IntoIter = std::slice::Iter<'a, StepWitness>;

    fn into_iter(self) -> Self::IntoIter {
        self.witnesses.iter()
    }
}

impl FromIterator<StepWitness> for WitnessBatch {
    fn from_iter<T: IntoIterator<Item = StepWitness>>(iter: T) -> Self {
        Self {
            witnesses: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_creation() {
        let transition = Transition::new(
            [1u8; 32],
            [2u8; 32],
            OriginClass::User,
            1000,
        );
        
        let witness = StepWitness::new(
            &transition,
            [0u8; 32],
            OriginClass::Genesis,
            0,
            vec![[0u8; 32]; 4],
            vec![false; 4],
            [0u8; 32],
            0,
            [0; 6],
            [1, u32::MAX, 10, 100, 5, 1000],
            [0u8; 32],
        );
        
        assert_eq!(witness.new_origin, OriginClass::User);
        assert_eq!(witness.prev_depth, 0);
        assert_eq!(witness.new_depth(), 1);
    }

    #[test]
    fn test_genesis_witness() {
        let witness = StepWitness::genesis(
            [42u8; 32],
            1000,
            [0u8; 32],
            vec![],
            vec![],
        );
        
        assert!(witness.is_genesis());
        assert_eq!(witness.prev_depth, 0);
        assert_eq!(witness.new_origin, OriginClass::Genesis);
    }

    #[test]
    fn test_witness_validation_valid() {
        let transition = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 1000);
        
        let valid_witness = StepWitness::new(
            &transition,
            [0u8; 32],
            OriginClass::Genesis,
            0,
            vec![[0u8; 32]; 4],
            vec![false; 4],
            [0u8; 32],
            0,
            [0; 6],
            [1, u32::MAX, 10, 100, 5, 1000],
            [0u8; 32],
        );
        
        assert!(valid_witness.validate_structure().is_ok());
    }

    #[test]
    fn test_witness_validation_invalid() {
        let transition = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 1000);
        
        // Invalid: mismatched proof/indices length
        let invalid_witness = StepWitness::new(
            &transition,
            [0u8; 32],
            OriginClass::Genesis,
            0,
            vec![[0u8; 32]; 4],
            vec![false; 3], // Wrong length!
            [0u8; 32],
            0,
            [0; 6],
            [1, u32::MAX, 10, 100, 5, 1000],
            [0u8; 32],
        );
        
        assert!(invalid_witness.validate_structure().is_err());
    }

    #[test]
    fn test_transition_hash_deterministic() {
        let transition = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 1000);
        
        let witness1 = StepWitness::new(
            &transition,
            [0u8; 32],
            OriginClass::Genesis,
            0,
            vec![],
            vec![],
            [0u8; 32],
            0,
            [0; 6],
            [0; 6],
            [0u8; 32],
        );
        
        let witness2 = StepWitness::new(
            &transition,
            [0u8; 32],
            OriginClass::Genesis,
            0,
            vec![],
            vec![],
            [0u8; 32],
            0,
            [0; 6],
            [0; 6],
            [0u8; 32],
        );
        
        assert_eq!(
            witness1.compute_transition_hash(),
            witness2.compute_transition_hash()
        );
    }

    #[test]
    fn test_counter_increment() {
        let transition = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 1000);
        
        let witness = StepWitness::new(
            &transition,
            [0u8; 32],
            OriginClass::User,
            5,
            vec![],
            vec![],
            [0u8; 32],
            0,
            [0, 10, 5, 0, 0, 0], // User=10, Admin=5
            [0; 6],
            [0u8; 32],
        );
        
        let new_counters = witness.compute_new_counters();
        
        // Admin counter should be incremented
        assert_eq!(new_counters[2], 6); // Admin was 5, now 6
        assert_eq!(new_counters[1], 10); // User unchanged
    }

    #[test]
    fn test_lineage_commitment_changes() {
        let transition = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 1000);
        
        let witness = StepWitness::new(
            &transition,
            [1u8; 32], // Non-zero previous commitment
            OriginClass::Genesis,
            5,
            vec![],
            vec![],
            [0u8; 32],
            0,
            [0; 6],
            [0; 6],
            [0u8; 32],
        );
        
        let new_commitment = witness.compute_new_lineage_commitment();
        
        // New commitment should be different from previous
        assert_ne!(new_commitment, witness.prev_lineage_commitment);
        assert_ne!(new_commitment, [0u8; 32]);
    }

    #[test]
    fn test_witness_batch() {
        let mut batch = WitnessBatch::new();
        
        assert!(batch.is_empty());
        
        let witness = StepWitness::default();
        batch.push(witness);
        
        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_witness_batch_iteration() {
        let witnesses: Vec<StepWitness> = (0..5)
            .map(|i| {
                let mut w = StepWitness::default();
                w.timestamp = i as u64 * 1000;
                w
            })
            .collect();
        
        let batch: WitnessBatch = witnesses.into_iter().collect();
        
        assert_eq!(batch.len(), 5);
        
        let timestamps: Vec<u64> = batch.iter().map(|w| w.timestamp).collect();
        assert_eq!(timestamps, vec![0, 1000, 2000, 3000, 4000]);
    }

    #[test]
    fn test_default_witness() {
        let witness = StepWitness::default();
        
        assert_eq!(witness.prev_depth, 0);
        assert_eq!(witness.new_origin, OriginClass::User);
        assert!(witness.policy_proof.is_empty());
    }
}
