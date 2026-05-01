//! Hashing utilities

use sha3::{Digest, Keccak256 as Sha3Keccak256};

/// Keccak256 hashing
pub struct Keccak256;

impl Keccak256 {
    /// Hash data with Keccak256
    pub fn hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3Keccak256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
    
    /// Hash to fixed-size array
    pub fn hash_to_array(data: &[u8]) -> [u8; 32] {
        let hash = Self::hash(data);
        let mut array = [0u8; 32];
        array.copy_from_slice(&hash[..32]);
        array
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keccak256() {
        let hash1 = Keccak256::hash(b"test");
        let hash2 = Keccak256::hash(b"test");
        assert_eq!(hash1, hash2);
    }
}