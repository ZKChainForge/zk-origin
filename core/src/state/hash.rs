// core/src/state/hash.rs

use sha3::Digest;

/// Keccak256 hasher
pub struct Keccak256Hasher(sha3::Keccak256);

impl Keccak256Hasher {
    /// Create new hasher
    pub fn new() -> Self {
        Keccak256Hasher(sha3::Keccak256::new())
    }
    
    /// Update with data
    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
    
    /// Finalize and get hash
    pub fn finalize(self) -> [u8; 32] {
        let result = self.0.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

impl Default for Keccak256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash a value with Keccak256
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

/// Hash state with metadata
pub fn hash_state(
    state_data: &[u8],
    timestamp: u64,
    nonce: u64,
) -> [u8; 32] {
    let mut hasher = Keccak256Hasher::new();
    hasher.update(state_data);
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&nonce.to_le_bytes());
    hasher.finalize()
}

/// Hash transition
pub fn hash_transition(
    prev_state: [u8; 32],
    new_state: [u8; 32],
    origin_class: u8,
    timestamp: u64,
    nonce: u64,
) -> [u8; 32] {
    let mut hasher = Keccak256Hasher::new();
    hasher.update(&prev_state);
    hasher.update(&new_state);
    hasher.update(&[origin_class]);
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&nonce.to_le_bytes());
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keccak256() {
        let hash1 = keccak256(b"test");
        let hash2 = keccak256(b"test");
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_different_inputs() {
        let hash1 = keccak256(b"test1");
        let hash2 = keccak256(b"test2");
        assert_ne!(hash1, hash2);
    }
}