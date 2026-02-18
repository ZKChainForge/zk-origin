//! Serialization utilities

use crate::{Result, ZkOriginError};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::Path;

/// Save a serializable object to JSON file
pub fn save_json<T: Serialize>(value: &T, path: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| ZkOriginError::SerializationError(e.to_string()))?;
    
    fs::write(path, json)?;
    Ok(())
}

/// Load a deserializable object from JSON file
pub fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let json = fs::read_to_string(path)?;
    serde_json::from_str(&json)
        .map_err(|e| ZkOriginError::SerializationError(e.to_string()))
}

/// Save to binary format
pub fn save_bincode<T: Serialize>(value: &T, path: impl AsRef<Path>) -> Result<()> {
    let bytes = bincode::serialize(value)?;
    fs::write(path, bytes)?;
    Ok(())
}

/// Load from binary format
pub fn load_bincode<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let bytes = fs::read(path)?;
    bincode::deserialize(&bytes)
        .map_err(|e| ZkOriginError::SerializationError(e.to_string()))
}

/// Convert bytes to hex string
pub fn to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hex string to bytes
pub fn from_hex(s: &str) -> Result<Vec<u8>> {
    hex::decode(s).map_err(|e| ZkOriginError::SerializationError(e.to_string()))
}

/// Convert 32-byte array to hex
pub fn bytes32_to_hex(bytes: &[u8; 32]) -> String {
    hex::encode(bytes)
}

/// Convert hex to 32-byte array
pub fn hex_to_bytes32(s: &str) -> Result<[u8; 32]> {
    let bytes = from_hex(s)?;
    if bytes.len() != 32 {
        return Err(ZkOriginError::SerializationError(
            format!("Expected 32 bytes, got {}", bytes.len())
        ));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
    struct TestStruct {
        value: u64,
        name: String,
    }

    #[test]
    fn test_json_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");

        let original = TestStruct {
            value: 42,
            name: "test".to_string(),
        };

        save_json(&original, &path).unwrap();
        let loaded: TestStruct = load_json(&path).unwrap();

        assert_eq!(original, loaded);
    }

    #[test]
    fn test_bincode_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.bin");

        let original = TestStruct {
            value: 42,
            name: "test".to_string(),
        };

        save_bincode(&original, &path).unwrap();
        let loaded: TestStruct = load_bincode(&path).unwrap();

        assert_eq!(original, loaded);
    }

    #[test]
    fn test_hex_conversion() {
        let bytes = [0xAB, 0xCD, 0xEF];
        let hex = to_hex(&bytes);
        assert_eq!(hex, "abcdef");

        let recovered = from_hex(&hex).unwrap();
        assert_eq!(recovered, bytes);
    }

    #[test]
    fn test_bytes32_hex() {
        let bytes = [42u8; 32];
        let hex = bytes32_to_hex(&bytes);
        let recovered = hex_to_bytes32(&hex).unwrap();
        assert_eq!(bytes, recovered);
    }
}