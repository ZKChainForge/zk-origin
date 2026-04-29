//! Utility functions for the prover

/// Format bytes as hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Format bytes as shortened hex string
pub fn bytes_to_hex_short(bytes: &[u8]) -> String {
    let display_len = 8.min(bytes.len());
    format!("0x{}...", hex::encode(&bytes[..display_len]))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bytes_to_hex() {
        let bytes = [1u8, 2, 3];
        assert_eq!(bytes_to_hex(&bytes), "010203");
    }
}