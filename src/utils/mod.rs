use sha2::{Sha256, Digest};
use hex;

/// Hash state data
pub fn hash_state(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("0x{}", hex::encode(result))
}

/// Convert bytes to hex string
pub fn to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Convert hex string to bytes
pub fn from_hex(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = hex_str.trim_start_matches("0x");
    hex::decode(clean)
        .map_err(|e| format!("Invalid hex: {}", e))
}

/// Validate hex string format
pub fn validate_hex(hex_str: &str, expected_len: usize) -> Result<(), String> {
    if !hex_str.starts_with("0x") {
        return Err("Hex string must start with 0x".to_string());
    }
    
    let clean = hex_str.trim_start_matches("0x");
    if clean.len() != expected_len * 2 {
        return Err(format!(
            "Expected {} bytes ({} hex chars), got {}",
            expected_len,
            expected_len * 2,
            clean.len()
        ));
    }
    
    Ok(())
}

/// Validate state hash (32 bytes = 64 hex chars)
pub fn validate_state_hash(hash: &str) -> Result<(), String> {
    validate_hex(hash, 32)
}

/// Validate origin class
pub fn validate_origin_class(class: u8) -> Result<(), String> {
    if class > 6 {
        return Err(format!("Invalid origin class: {}", class));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_state() {
        let data = b"test_state";
        let hash = hash_state(data);
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_hex_conversion() {
        let original = vec![0x12, 0x34, 0x56, 0x78];
        let hex = to_hex(&original);
        assert_eq!(hex, "0x12345678");

        let recovered = from_hex(&hex).unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_validate_state_hash() {
        let valid = "0x0000000000000000000000000000000000000000000000000000000000000000";
        assert!(validate_state_hash(valid).is_ok());

        let invalid = "0x000000000000000000000000000000000000000000000000000000000000000";
        assert!(validate_state_hash(invalid).is_err());
    }
}