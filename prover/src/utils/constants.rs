//! System constants

/// Number of origin classes
pub const NUM_ORIGIN_CLASSES: usize = 6;

/// Policy Merkle tree depth
pub const POLICY_TREE_DEPTH: usize = 4;

/// Maximum lineage depth
pub const MAX_LINEAGE_DEPTH: u64 = 1_000_000;

/// Default epoch duration (24 hours in seconds)
pub const DEFAULT_EPOCH_DURATION: u64 = 86400;

/// Proof version marker
pub const PROOF_VERSION: u8 = 1;

/// Circuit identifier
pub const CIRCUIT_ID: &str = "zk-origin-v1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(NUM_ORIGIN_CLASSES, 6);
        assert_eq!(POLICY_TREE_DEPTH, 4);
        assert!(MAX_LINEAGE_DEPTH > 0);
    }
}