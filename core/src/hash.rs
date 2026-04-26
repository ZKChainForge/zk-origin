// core/src/hash.rs

use sha3::Digest;

/// Hash data with Keccak256
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = sha3::Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Hash state with metadata
pub fn hash_state(
    state_data: &[u8],
    timestamp: u64,
    nonce: u64,
) -> [u8; 32] {
    let mut hasher = sha3::Keccak256::new();
    hasher.update(state_data);
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&nonce.to_le_bytes());
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Hash transition
pub fn hash_transition(
    prev_state: [u8; 32],
    new_state: [u8; 32],
    origin_class: u8,
    timestamp: u64,
    nonce: u64,
) -> [u8; 32] {
    let mut hasher = sha3::Keccak256::new();
    hasher.update(&prev_state);
    hasher.update(&new_state);
    hasher.update(&[origin_class]);
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&nonce.to_le_bytes());
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
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
}