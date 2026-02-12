//! Hashing utilities

/// Convert bytes to field element representation
pub fn bytes_to_field(bytes: &[u8; 32]) -> [u64; 4] {
    let mut result = [0u64; 4];
    for i in 0..4 {
        let start = i * 8;
        let end = start + 8;
        result[i] = u64::from_le_bytes(bytes[start..end].try_into().unwrap());
    }
    result
}

/// Convert field element representation to bytes
pub fn field_to_bytes(field: &[u64; 4]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..4 {
        let bytes = field[i].to_le_bytes();
        result[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_round_trip() {
        let original = [42u8; 32];
        let field = bytes_to_field(&original);
        let recovered = field_to_bytes(&field);
        assert_eq!(original, recovered);
    }
}