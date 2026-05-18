//! Cryptographic hashing utilities for Nova proofs

use crate::error::{NovaError, Result};
use blake3::Hasher as Blake3Hasher;
use sha3::{Digest, Sha3_256};
use serde::{Deserialize, Serialize};

/// Hash algorithm types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HashType {
    /// SHA3-256 algorithm
    SHA3_256,
    /// BLAKE3 algorithm
    BLAKE3,
}

/// Hash output (256 bits / 32 bytes)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create a hash from a 32-byte array
    pub fn from_array(arr: [u8; 32]) -> Self {
        Hash(arr)
    }

    /// Get the hash as a 32-byte array reference
    pub fn as_array(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get the hash as a byte slice
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Convert hash to hexadecimal string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Create hash from hexadecimal string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|e| NovaError::Other(e.to_string()))?;

        if bytes.len() != 32 {
            return Err(NovaError::InvalidProofData {
                context: format!("Expected 32 bytes, got {}", bytes.len()),
            });
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash(arr))
    }

    /// Check if hash is all zeros
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash([0u8; 32])
    }
}

/// Cryptographic hasher for incremental hashing
pub struct Hasher {
    hash_type: HashType,
    sha3_hasher: Option<Sha3_256>,
    blake3_hasher: Option<Blake3Hasher>,
}

impl Hasher {
    /// Create a new hasher with the specified hash type
    pub fn new(hash_type: HashType) -> Self {
        match hash_type {
            HashType::SHA3_256 => Hasher {
                hash_type,
                sha3_hasher: Some(Sha3_256::new()),
                blake3_hasher: None,
            },
            HashType::BLAKE3 => Hasher {
                hash_type,
                sha3_hasher: None,
                blake3_hasher: Some(Blake3Hasher::new()),
            },
        }
    }

    /// Update the hasher with additional data
    pub fn update(&mut self, data: &[u8]) {
        match self.hash_type {
            HashType::SHA3_256 => {
                if let Some(ref mut h) = self.sha3_hasher {
                    h.update(data);
                }
            }
            HashType::BLAKE3 => {
                if let Some(ref mut h) = self.blake3_hasher {
                    h.update(data);
                }
            }
        }
    }

    /// Finalize the hash and consume the hasher
    pub fn finalize(self) -> Hash {
        match self.hash_type {
            HashType::SHA3_256 => {
                let result = self.sha3_hasher.unwrap().finalize();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&result);
                Hash(arr)
            }
            HashType::BLAKE3 => {
                let result = self.blake3_hasher.unwrap().finalize();
                Hash(*result.as_bytes())
            }
        }
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new(HashType::SHA3_256)
    }
}

/// Hash data with SHA3-256
pub fn sha3_256(data: &[u8]) -> Hash {
    let mut hasher = Hasher::new(HashType::SHA3_256);
    hasher.update(data);
    hasher.finalize()
}

/// Hash data with BLAKE3
pub fn blake3(data: &[u8]) -> Hash {
    let mut hasher = Hasher::new(HashType::BLAKE3);
    hasher.update(data);
    hasher.finalize()
}

/// Hash multiple inputs sequentially
pub fn hash_multi(inputs: &[&[u8]]) -> Hash {
    let mut hasher = Hasher::new(HashType::SHA3_256);
    for input in inputs {
        hasher.update(input);
    }
    hasher.finalize()
}

/// Verify hash consistency
pub fn verify_hash(data: &[u8], expected: Hash) -> bool {
    sha3_256(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let h1 = sha3_256(b"test");
        let h2 = sha3_256(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_inputs() {
        let h1 = sha3_256(b"test1");
        let h2 = sha3_256(b"test2");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_hex_conversion() {
        let h = sha3_256(b"test");
        let hex = h.to_hex();
        let h2 = Hash::from_hex(&hex).unwrap();
        assert_eq!(h, h2);
    }

    #[test]
    fn test_blake3_hash() {
        let h1 = blake3(b"test");
        let h2 = blake3(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_zero() {
        let h = Hash::default();
        assert!(h.is_zero());
    }

    #[test]
    fn test_multi_hash() {
        let result = hash_multi(&[b"a", b"b", b"c"]);
        assert!(!result.is_zero());
    }
}