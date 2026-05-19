
pub mod poseidon;

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt;

/// 32-byte hash type (SHA3-256 output)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create hash from byte array
    pub fn from_array(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    /// Create hash from slice (will panic if wrong length)
    pub fn from_slice(slice: &[u8]) -> Self {
        assert_eq!(slice.len(), 32, "Hash slice must be 32 bytes");
        let mut arr = [0u8; 32];
        arr.copy_from_slice(slice);
        Hash(arr)
    }

    /// Get as byte slice
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Get as mutable byte slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }

    /// Get as byte array
    pub fn as_array(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash(arr))
    }

    /// Check if hash is zero
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash([0u8; 32])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }
}

impl From<&[u8; 32]> for Hash {
    fn from(bytes: &[u8; 32]) -> Self {
        Hash(*bytes)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// SHA3-256 hash function
pub fn sha3_256(data: &[u8]) -> Hash {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    let result = hasher.finalize();
    
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    Hash(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_from_array() {
        let arr = [1u8; 32];
        let hash = Hash::from_array(arr);
        assert_eq!(hash.as_array(), &arr);
    }

    #[test]
    fn test_hash_to_hex() {
        let arr = [0xFFu8; 32];
        let hash = Hash::from_array(arr);
        let hex = hash.to_hex();
        assert_eq!(hex.len(), 64);
    }

    #[test]
    fn test_hash_from_hex() {
        let hex = "ff".repeat(32);
        let hash = Hash::from_hex(&hex).unwrap();
        assert_eq!(hash.as_array(), &[0xFFu8; 32]);
    }

    #[test]
    fn test_sha3_256() {
        let data = b"test data";
        let hash = sha3_256(data);
        let hash2 = sha3_256(data);
        assert_eq!(hash, hash2);
    }
}