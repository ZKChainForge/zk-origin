//! Type conversion utilities

use crate::types::OriginClass;

/// Convert u64 to 32-byte array (little-endian, zero-padded)
pub fn u64_to_bytes32(value: u64) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&value.to_le_bytes());
    bytes
}

/// Convert 32-byte array to u64 (reads first 8 bytes)
pub fn bytes32_to_u64(bytes: &[u8; 32]) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(arr)
}

/// Convert u32 array to bytes
pub fn u32_array_to_bytes(arr: &[u32]) -> Vec<u8> {
    arr.iter()
        .flat_map(|&v| v.to_le_bytes())
        .collect()
}

/// Convert bytes to u32 array
pub fn bytes_to_u32_array(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks(4)
        .map(|chunk| {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(chunk);
            u32::from_le_bytes(arr)
        })
        .collect()
}

/// Convert origin class array to bytes
pub fn origins_to_bytes(origins: &[OriginClass]) -> Vec<u8> {
    origins.iter().map(|o| *o as u8).collect()
}

/// Convert bytes to origin class array
pub fn bytes_to_origins(bytes: &[u8]) -> Vec<Option<OriginClass>> {
    bytes.iter().map(|&b| OriginClass::try_from_u8(b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_conversion() {
        let value = 0x123456789ABCDEF0u64;
        let bytes = u64_to_bytes32(value);
        let recovered = bytes32_to_u64(&bytes);
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_u32_array_conversion() {
        let arr = [1u32, 2, 3, 4];
        let bytes = u32_array_to_bytes(&arr);
        let recovered = bytes_to_u32_array(&bytes);
        assert_eq!(arr.to_vec(), recovered);
    }

    #[test]
    fn test_origin_conversion() {
        let origins = vec![OriginClass::Genesis, OriginClass::User, OriginClass::Admin];
        let bytes = origins_to_bytes(&origins);
        let recovered: Vec<_> = bytes_to_origins(&bytes).into_iter().flatten().collect();
        assert_eq!(origins, recovered);
    }
}