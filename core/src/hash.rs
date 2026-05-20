//! Uses Keccak256 (same as Ethereum) for compatibility

use sha3::Digest;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a 256-bit hash
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create from array
    pub fn from_array(arr: [u8; 32]) -> Self {
        Hash(arr)
    }

    /// Get as array reference
    pub fn as_array(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get as slice
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let bytes = hex::decode(hex_str).map_err(|e| e.to_string())?;
        if bytes.len() != 32 {
            return Err(format!("Expected 32 bytes, got {}", bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash(arr))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash([0u8; 32])
    }
}

/// Keccak256 hasher wrapper
pub struct Hasher {
    hasher: sha3::Keccak256,
}

impl Hasher {
    /// Create new hasher
    pub fn new() -> Self {
        Hasher {
            hasher: sha3::Keccak256::new(),
        }
    }

    /// Update with data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize hash
    pub fn finalize(self) -> Hash {
        let result = self.hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        Hash(arr)
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash arbitrary data with Keccak256
pub fn keccak256(data: &[u8]) -> Hash {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

/// Hash state with metadata
pub fn hash_state(state_data: &[u8], timestamp: u64, nonce: u64) -> Hash {
    let mut hasher = Hasher::new();
    hasher.update(state_data);
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&nonce.to_le_bytes());
    hasher.finalize()
}

/// Hash transition
pub fn hash_transition(
    prev_state: Hash,
    new_state: Hash,
    origin_class: u8,
    timestamp: u64,
    nonce: u64,
) -> Hash {
    let mut hasher = Hasher::new();
    hasher.update(prev_state.as_slice());
    hasher.update(new_state.as_slice());
    hasher.update(&[origin_class]);
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&nonce.to_le_bytes());
    hasher.finalize()
}

/// Hash lineage commitment
pub fn hash_lineage(prev_commitment: Hash, transition_hash: Hash, depth: u32) -> Hash {
    let mut hasher = Hasher::new();
    hasher.update(prev_commitment.as_slice());
    hasher.update(transition_hash.as_slice());
    hasher.update(&depth.to_le_bytes());
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let h1 = keccak256(b"test");
        let h2 = keccak256(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_inputs() {
        let h1 = keccak256(b"test1");
        let h2 = keccak256(b"test2");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_hex_conversion() {
        let h = keccak256(b"test");
        let hex = h.to_hex();
        let h2 = Hash::from_hex(&hex).unwrap();
        assert_eq!(h, h2);
    }

    #[test]
    fn test_hash_display() {
        let h = keccak256(b"test");
        let display = format!("{}", h);
        assert!(display.starts_with("0x"));
    }
}