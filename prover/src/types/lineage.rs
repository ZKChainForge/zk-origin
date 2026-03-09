//! Lineage commitment types

use serde::{Deserialize, Serialize};
use std::fmt;

/// A cryptographic commitment to a state's complete lineage.
///
/// The lineage commitment is computed recursively:
/// - Genesis: C₀ = Hash(genesis_state, 0, 0)
/// - Step n: Cₙ = Hash(Cₙ₋₁, transition_hash, n)
///
/// This allows us to commit to the entire history in constant space.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineageCommitment {
    /// The commitment value (Poseidon hash output)
    pub value: [u8; 32],

    /// The depth of the lineage (number of transitions from genesis)
    pub depth: u64,
}

impl LineageCommitment {
    /// Create a new lineage commitment
    pub fn new(value: [u8; 32], depth: u64) -> Self {
        Self { value, depth }
    }

    /// Create the genesis lineage commitment
    pub fn genesis(genesis_state_hash: [u8; 32]) -> Self {
        // For genesis, we hash: (state_hash, 0, 0)
        // In practice, this would use Poseidon hash
        // For now, we use a placeholder
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(genesis_state_hash);
        hasher.update([0u8; 8]); // origin = 0
        hasher.update([0u8; 8]); // depth = 0

        let result = hasher.finalize();
        let mut value = [0u8; 32];
        value.copy_from_slice(&result);

        Self { value, depth: 0 }
    }

    /// Create a zero/empty commitment
    pub fn zero() -> Self {
        Self {
            value: [0u8; 32],
            depth: 0,
        }
    }

    /// Check if this is the genesis commitment
    pub fn is_genesis(&self) -> bool {
        self.depth == 0
    }

    /// Get the commitment as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.value
    }

    /// Get the commitment as a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.value)
    }

    /// Create from hex string
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut value = [0u8; 32];
        value.copy_from_slice(&bytes);
        Ok(Self { value, depth: 0 })
    }
}

impl fmt::Debug for LineageCommitment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineageCommitment")
            .field("value", &self.to_hex())
            .field("depth", &self.depth)
            .finish()
    }
}

impl fmt::Display for LineageCommitment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}...@{}", &self.to_hex()[..8], self.depth)
    }
}

impl Default for LineageCommitment {
    fn default() -> Self {
        Self::zero()
    }
}

/// Counter commitment for rate limiting
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CounterCommitment {
    /// The commitment value
    pub value: [u8; 32],

    /// The epoch this counter is for
    pub epoch: u64,
}

impl CounterCommitment {
    /// Create a new counter commitment
    pub fn new(value: [u8; 32], epoch: u64) -> Self {
        Self { value, epoch }
    }

    /// Create initial counter commitment for an epoch
    pub fn initial(epoch: u64) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(epoch.to_le_bytes());
        // All counters start at 0
        for _ in 0..6 {
            hasher.update(0u32.to_le_bytes());
        }

        let result = hasher.finalize();
        let mut value = [0u8; 32];
        value.copy_from_slice(&result);

        Self { value, epoch }
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.value)
    }
}

impl fmt::Debug for CounterCommitment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CounterCommitment")
            .field("value", &self.to_hex())
            .field("epoch", &self.epoch)
            .finish()
    }
}

impl Default for CounterCommitment {
    fn default() -> Self {
        Self::initial(0)
    }
}

/// Epoch counters tracking usage per origin class
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochCounters {
    /// Current epoch
    pub epoch: u64,

    /// Counter for each origin class
    pub counts: [u32; 6],
}

impl EpochCounters {
    /// Create new counters for an epoch
    pub fn new(epoch: u64) -> Self {
        Self {
            epoch,
            counts: [0; 6],
        }
    }

    /// Increment counter for an origin class
    pub fn increment(&mut self, origin: super::OriginClass) {
        let idx = origin as usize;
        if idx < self.counts.len() {
            self.counts[idx] = self.counts[idx].saturating_add(1);
        }
    }

    /// Get counter for an origin class
    pub fn get(&self, origin: super::OriginClass) -> u32 {
        let idx = origin as usize;
        if idx < self.counts.len() {
            self.counts[idx]
        } else {
            0
        }
    }

    /// Check if rate limit would be exceeded
    pub fn would_exceed_limit(&self, origin: super::OriginClass, limit: u32) -> bool {
        self.get(origin) >= limit
    }

    /// Compute commitment for these counters
    pub fn compute_commitment(&self) -> CounterCommitment {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.epoch.to_le_bytes());
        for count in &self.counts {
            hasher.update(count.to_le_bytes());
        }

        let result = hasher.finalize();
        let mut value = [0u8; 32];
        value.copy_from_slice(&result);

        CounterCommitment::new(value, self.epoch)
    }
}

impl Default for EpochCounters {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OriginClass;

    #[test]
    fn test_lineage_commitment_genesis() {
        let state_hash = [1u8; 32];
        let commitment = LineageCommitment::genesis(state_hash);

        assert!(commitment.is_genesis());
        assert_eq!(commitment.depth, 0);
    }

    #[test]
    fn test_lineage_commitment_hex() {
        let commitment = LineageCommitment::new([0xAB; 32], 5);
        let hex = commitment.to_hex();

        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_epoch_counters() {
        let mut counters = EpochCounters::new(42);

        assert_eq!(counters.get(OriginClass::User), 0);

        counters.increment(OriginClass::User);
        counters.increment(OriginClass::User);
        counters.increment(OriginClass::Admin);

        assert_eq!(counters.get(OriginClass::User), 2);
        assert_eq!(counters.get(OriginClass::Admin), 1);
        assert_eq!(counters.get(OriginClass::Bridge), 0);
    }

    #[test]
    fn test_rate_limit_check() {
        let mut counters = EpochCounters::new(0);

        // Admin limit is 10
        for _ in 0..10 {
            assert!(!counters.would_exceed_limit(OriginClass::Admin, 10));
            counters.increment(OriginClass::Admin);
        }

        // Now at limit
        assert!(counters.would_exceed_limit(OriginClass::Admin, 10));
    }

    #[test]
    fn test_counter_commitment_deterministic() {
        let counters1 = EpochCounters::new(42);
        let counters2 = EpochCounters::new(42);

        assert_eq!(
            counters1.compute_commitment().value,
            counters2.compute_commitment().value
        );
    }
}
