//! Poseidon hash implementation
//!
//! This module provides both native (off-chain) and circuit (in-proof)
//! Poseidon hash implementations.

use sha2::{Sha256, Digest};
use std::marker::PhantomData;

/// Width of Poseidon hash (number of inputs + 1 for capacity)
pub const POSEIDON_WIDTH: usize = 3;

/// Number of full rounds
pub const POSEIDON_FULL_ROUNDS: usize = 8;

/// Number of partial rounds
pub const POSEIDON_PARTIAL_ROUNDS: usize = 57;

/// Poseidon hasher for native (off-chain) computation
/// 
/// Note: This is a placeholder implementation using SHA256.
/// In production, this would use the actual Poseidon algorithm
/// with proper round constants for the target field.
#[derive(Clone, Debug)]
pub struct PoseidonHasher {
    _phantom: PhantomData<()>,
}

impl PoseidonHasher {
    /// Create a new Poseidon hasher
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Hash a variable number of inputs
    pub fn hash(&self, inputs: &[[u8; 32]]) -> [u8; 32] {
        // Placeholder: Use SHA256 as stand-in for Poseidon
        // In production, this would use actual Poseidon
        let mut hasher = Sha256::new();
        
        // Domain separation
        hasher.update(b"poseidon");
        hasher.update(&(inputs.len() as u64).to_le_bytes());
        
        for input in inputs {
            hasher.update(input);
        }
        
        let result = hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(&result);
        output
    }

    /// Hash two inputs
    pub fn hash_two(&self, left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        self.hash(&[*left, *right])
    }

    /// Hash three inputs
    pub fn hash_three(&self, a: &[u8; 32], b: &[u8; 32], c: &[u8; 32]) -> [u8; 32] {
        self.hash(&[*a, *b, *c])
    }

    /// Hash five inputs (for transition hash)
    pub fn hash_five(
        &self,
        a: &[u8; 32],
        b: &[u8; 32],
        c: &[u8; 32],
        d: &[u8; 32],
        e: &[u8; 32],
    ) -> [u8; 32] {
        self.hash(&[*a, *b, *c, *d, *e])
    }

    /// Hash a u64 value (converts to 32 bytes)
    pub fn hash_u64(&self, value: u64) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&value.to_le_bytes());
        self.hash(&[bytes])
    }

    /// Hash bytes of arbitrary length
    pub fn hash_bytes(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"poseidon_bytes");
        hasher.update(data);
        
        let result = hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(&result);
        output
    }
}

impl Default for PoseidonHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for hashing with default hasher
pub fn poseidon_hash(inputs: &[[u8; 32]]) -> [u8; 32] {
    PoseidonHasher::new().hash(inputs)
}

/// Convenience function for hashing two inputs
pub fn poseidon_hash_two(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    PoseidonHasher::new().hash_two(left, right)
}

/// Compute lineage commitment from previous commitment and transition
pub fn compute_lineage_commitment(
    prev_commitment: &[u8; 32],
    transition_hash: &[u8; 32],
    depth: u64,
) -> [u8; 32] {
    let hasher = PoseidonHasher::new();
    
    let mut depth_bytes = [0u8; 32];
    depth_bytes[..8].copy_from_slice(&depth.to_le_bytes());
    
    hasher.hash_three(prev_commitment, transition_hash, &depth_bytes)
}

/// Compute transition hash
pub fn compute_transition_hash(
    prev_state: &[u8; 32],
    new_state: &[u8; 32],
    origin: u8,
    timestamp: u64,
    epoch: u64,
) -> [u8; 32] {
    let hasher = PoseidonHasher::new();
    
    let mut origin_bytes = [0u8; 32];
    origin_bytes[0] = origin;
    
    let mut timestamp_bytes = [0u8; 32];
    timestamp_bytes[..8].copy_from_slice(&timestamp.to_le_bytes());
    
    let mut epoch_bytes = [0u8; 32];
    epoch_bytes[..8].copy_from_slice(&epoch.to_le_bytes());
    
    hasher.hash_five(prev_state, new_state, &origin_bytes, &timestamp_bytes, &epoch_bytes)
}

/// Compute counter commitment
pub fn compute_counter_commitment(epoch: u64, counters: &[u32; 6]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"counter_commitment");
    hasher.update(&epoch.to_le_bytes());
    
    for counter in counters {
        hasher.update(&counter.to_le_bytes());
    }
    
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}


/// Compute policy leaf hash
pub fn compute_policy_leaf(from_origin: u8, to_origin: u8) -> [u8; 32] {
    let hasher = PoseidonHasher::new();
    
    let mut from_bytes = [0u8; 32];
    from_bytes[0] = from_origin;
    
    let mut to_bytes = [0u8; 32];
    to_bytes[0] = to_origin;
    
    hasher.hash_two(&from_bytes, &to_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poseidon_deterministic() {
        let hasher = PoseidonHasher::new();
        
        let input1 = [1u8; 32];
        let input2 = [2u8; 32];
        
        let hash1 = hasher.hash_two(&input1, &input2);
        let hash2 = hasher.hash_two(&input1, &input2);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_different_inputs() {
        let hasher = PoseidonHasher::new();
        
        let input1 = [1u8; 32];
        let input2 = [2u8; 32];
        let input3 = [3u8; 32];
        
        let hash1 = hasher.hash_two(&input1, &input2);
        let hash2 = hasher.hash_two(&input1, &input3);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_order_matters() {
        let hasher = PoseidonHasher::new();
        
        let input1 = [1u8; 32];
        let input2 = [2u8; 32];
        
        let hash1 = hasher.hash_two(&input1, &input2);
        let hash2 = hasher.hash_two(&input2, &input1);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_lineage_commitment() {
        let prev = [1u8; 32];
        let transition = [2u8; 32];
        
        let commit1 = compute_lineage_commitment(&prev, &transition, 5);
        let commit2 = compute_lineage_commitment(&prev, &transition, 5);
        let commit3 = compute_lineage_commitment(&prev, &transition, 6);
        
        assert_eq!(commit1, commit2);
        assert_ne!(commit1, commit3);
    }

    #[test]
    fn test_transition_hash() {
        let prev = [1u8; 32];
        let new = [2u8; 32];
        
        let hash1 = compute_transition_hash(&prev, &new, 1, 1000, 0);
        let hash2 = compute_transition_hash(&prev, &new, 1, 1000, 0);
        let hash3 = compute_transition_hash(&prev, &new, 2, 1000, 0);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_counter_commitment() {
        let counters1 = [0, 10, 5, 0, 0, 0];
        let counters2 = [0, 10, 6, 0, 0, 0];
        
        let commit1 = compute_counter_commitment(42, &counters1);
        let commit2 = compute_counter_commitment(42, &counters1);
        let commit3 = compute_counter_commitment(42, &counters2);
        
        assert_eq!(commit1, commit2);
        assert_ne!(commit1, commit3);
    }

    #[test]
    fn test_policy_leaf() {
        let leaf1 = compute_policy_leaf(1, 2);
        let leaf2 = compute_policy_leaf(1, 2);
        let leaf3 = compute_policy_leaf(2, 1);
        
        assert_eq!(leaf1, leaf2);
        assert_ne!(leaf1, leaf3);
    }

    #[test]
    fn test_convenience_functions() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        
        let hash1 = poseidon_hash(&[a, b]);
        let hash2 = poseidon_hash_two(&a, &b);
        
        assert_eq!(hash1, hash2);
    }
}