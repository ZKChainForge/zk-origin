//! Origin policy definitions

use super::OriginClass;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Defines the allowed transitions between origin classes.
///
/// The policy acts as a firewall, controlling which state transitions
/// are permitted based on the origin class of the previous and new states.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OriginPolicy {
    /// Set of allowed (from, to) origin class pairs
    allowed_transitions: HashSet<(OriginClass, OriginClass)>,

    /// Rate limits per origin class per epoch
    rate_limits: [u32; 6],

    /// Epoch duration in seconds
    pub epoch_duration: u64,

    /// Policy version for upgrades
    pub version: u32,
}

impl OriginPolicy {
    /// Create a new empty policy (no transitions allowed)
    pub fn new_empty(epoch_duration: u64) -> Self {
        Self {
            allowed_transitions: HashSet::new(),
            rate_limits: [0; 6],
            epoch_duration,
            version: 1,
        }
    }

    /// Create the default permissive policy
    pub fn default_permissive() -> Self {
        let mut policy = Self::new_empty(86400); // 24 hour epochs

        // Genesis can go to User or Admin
        policy.allow(OriginClass::Genesis, OriginClass::User);
        policy.allow(OriginClass::Genesis, OriginClass::Admin);
        policy.allow(OriginClass::Genesis, OriginClass::System);

        // User can only go to User
        policy.allow(OriginClass::User, OriginClass::User);

        // Admin can go to User, Admin, Bridge, or System
        policy.allow(OriginClass::Admin, OriginClass::User);
        policy.allow(OriginClass::Admin, OriginClass::Admin);
        policy.allow(OriginClass::Admin, OriginClass::Bridge);
        policy.allow(OriginClass::Admin, OriginClass::System);

        // Bridge can only go to User
        policy.allow(OriginClass::Bridge, OriginClass::User);

        // Governance can go anywhere
        for to in OriginClass::all() {
            if *to != OriginClass::Genesis {
                policy.allow(OriginClass::Governance, *to);
            }
        }

        // System can go to User or System
        policy.allow(OriginClass::System, OriginClass::User);
        policy.allow(OriginClass::System, OriginClass::System);

        // Set default rate limits
        policy.rate_limits = [
            1,        // Genesis: 1
            u32::MAX, // User: unlimited
            10,       // Admin: 10 per epoch
            100,      // Bridge: 100 per epoch
            5,        // Governance: 5 per epoch
            1000,     // System: 1000 per epoch
        ];

        policy
    }

    /// Create a restrictive policy (for testing)
    pub fn restrictive() -> Self {
        let mut policy = Self::new_empty(86400);

        // Only allow: Genesis → User → User
        policy.allow(OriginClass::Genesis, OriginClass::User);
        policy.allow(OriginClass::User, OriginClass::User);

        policy.rate_limits = [1, 100, 0, 0, 0, 0];

        policy
    }

    /// Allow a transition from one origin class to another
    pub fn allow(&mut self, from: OriginClass, to: OriginClass) {
        self.allowed_transitions.insert((from, to));
    }

    /// Disallow a transition
    pub fn disallow(&mut self, from: OriginClass, to: OriginClass) {
        self.allowed_transitions.remove(&(from, to));
    }

    /// Check if a transition is allowed
    pub fn is_allowed(&self, from: OriginClass, to: OriginClass) -> bool {
        self.allowed_transitions.contains(&(from, to))
    }

    /// Set rate limit for an origin class
    pub fn set_rate_limit(&mut self, origin: OriginClass, limit: u32) {
        let idx = origin as usize;
        if idx < self.rate_limits.len() {
            self.rate_limits[idx] = limit;
        }
    }

    /// Get rate limit for an origin class
    pub fn get_rate_limit(&self, origin: OriginClass) -> u32 {
        let idx = origin as usize;
        if idx < self.rate_limits.len() {
            self.rate_limits[idx]
        } else {
            0
        }
    }

    /// Get all allowed transitions
    pub fn allowed_transitions(&self) -> &HashSet<(OriginClass, OriginClass)> {
        &self.allowed_transitions
    }

    /// Get the number of allowed transitions
    pub fn num_allowed(&self) -> usize {
        self.allowed_transitions.len()
    }

    /// Compute policy hash for verification
    pub fn compute_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.version.to_le_bytes());
        hasher.update(self.epoch_duration.to_le_bytes());

        // Hash allowed transitions in deterministic order
        let mut transitions: Vec<_> = self.allowed_transitions.iter().collect();
        transitions.sort_by_key(|(f, t)| (*f as u8, *t as u8));

        for (from, to) in transitions {
            hasher.update([*from as u8, *to as u8]);
        }

        // Hash rate limits
        for limit in &self.rate_limits {
            hasher.update(limit.to_le_bytes());
        }

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Get all transitions from a given origin class
    pub fn transitions_from(&self, from: OriginClass) -> Vec<OriginClass> {
        self.allowed_transitions
            .iter()
            .filter(|(f, _)| *f == from)
            .map(|(_, t)| *t)
            .collect()
    }

    /// Get all transitions to a given origin class
    pub fn transitions_to(&self, to: OriginClass) -> Vec<OriginClass> {
        self.allowed_transitions
            .iter()
            .filter(|(_, t)| *t == to)
            .map(|(f, _)| *f)
            .collect()
    }

    /// Convert to adjacency matrix representation
    pub fn to_adjacency_matrix(&self) -> [[bool; 6]; 6] {
        let mut matrix = [[false; 6]; 6];
        for (from, to) in &self.allowed_transitions {
            matrix[*from as usize][*to as usize] = true;
        }
        matrix
    }

    /// Create from adjacency matrix
    pub fn from_adjacency_matrix(matrix: [[bool; 6]; 6], epoch_duration: u64) -> Self {
        let mut policy = Self::new_empty(epoch_duration);

        for (from_idx, row) in matrix.iter().enumerate() {
            for (to_idx, &allowed) in row.iter().enumerate() {
                if allowed {
                    if let (Some(from), Some(to)) = (
                        OriginClass::try_from_u8(from_idx as u8),
                        OriginClass::try_from_u8(to_idx as u8),
                    ) {
                        policy.allow(from, to);
                    }
                }
            }
        }

        policy
    }
}

impl Default for OriginPolicy {
    fn default() -> Self {
        Self::default_permissive()
    }
}

impl PartialEq for OriginPolicy {
    fn eq(&self, other: &Self) -> bool {
        self.allowed_transitions == other.allowed_transitions
            && self.rate_limits == other.rate_limits
            && self.epoch_duration == other.epoch_duration
            && self.version == other.version
    }
}

impl Eq for OriginPolicy {}

/// Builder for creating policies
#[derive(Default)]
pub struct PolicyBuilder {
    policy: OriginPolicy,
}

impl PolicyBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            policy: OriginPolicy::new_empty(86400),
        }
    }

    /// Set epoch duration
    pub fn epoch_duration(mut self, duration: u64) -> Self {
        self.policy.epoch_duration = duration;
        self
    }

    /// Allow a transition
    pub fn allow(mut self, from: OriginClass, to: OriginClass) -> Self {
        self.policy.allow(from, to);
        self
    }

    /// Set rate limit
    pub fn rate_limit(mut self, origin: OriginClass, limit: u32) -> Self {
        self.policy.set_rate_limit(origin, limit);
        self
    }

    /// Build the policy
    pub fn build(self) -> OriginPolicy {
        self.policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = OriginPolicy::default();

        // Check some expected transitions
        assert!(policy.is_allowed(OriginClass::Genesis, OriginClass::User));
        assert!(policy.is_allowed(OriginClass::User, OriginClass::User));
        assert!(policy.is_allowed(OriginClass::Admin, OriginClass::User));

        // Check some forbidden transitions
        assert!(!policy.is_allowed(OriginClass::User, OriginClass::Admin));
        assert!(!policy.is_allowed(OriginClass::Bridge, OriginClass::Admin));
    }

    #[test]
    fn test_restrictive_policy() {
        let policy = OriginPolicy::restrictive();

        assert!(policy.is_allowed(OriginClass::Genesis, OriginClass::User));
        assert!(policy.is_allowed(OriginClass::User, OriginClass::User));
        assert!(!policy.is_allowed(OriginClass::User, OriginClass::Admin));
        assert!(!policy.is_allowed(OriginClass::Admin, OriginClass::User));
    }

    #[test]
    fn test_policy_modification() {
        let mut policy = OriginPolicy::new_empty(3600);

        assert!(!policy.is_allowed(OriginClass::User, OriginClass::Admin));

        policy.allow(OriginClass::User, OriginClass::Admin);
        assert!(policy.is_allowed(OriginClass::User, OriginClass::Admin));

        policy.disallow(OriginClass::User, OriginClass::Admin);
        assert!(!policy.is_allowed(OriginClass::User, OriginClass::Admin));
    }

    #[test]
    fn test_rate_limits() {
        let policy = OriginPolicy::default();

        assert_eq!(policy.get_rate_limit(OriginClass::Genesis), 1);
        assert_eq!(policy.get_rate_limit(OriginClass::User), u32::MAX);
        assert_eq!(policy.get_rate_limit(OriginClass::Admin), 10);
    }

    #[test]
    fn test_policy_hash_deterministic() {
        let policy1 = OriginPolicy::default();
        let policy2 = OriginPolicy::default();

        assert_eq!(policy1.compute_hash(), policy2.compute_hash());
    }

    #[test]
    fn test_policy_hash_changes() {
        let mut policy1 = OriginPolicy::default();
        let policy2 = OriginPolicy::default();

        policy1.allow(OriginClass::Bridge, OriginClass::Admin);

        assert_ne!(policy1.compute_hash(), policy2.compute_hash());
    }

    #[test]
    fn test_adjacency_matrix() {
        let policy = OriginPolicy::restrictive();
        let matrix = policy.to_adjacency_matrix();

        assert!(matrix[0][1]); // Genesis → User
        assert!(matrix[1][1]); // User → User
        assert!(!matrix[1][2]); // User → Admin (not allowed)
    }

    #[test]
    fn test_policy_builder() {
        let policy = PolicyBuilder::new()
            .epoch_duration(3600)
            .allow(OriginClass::Genesis, OriginClass::User)
            .allow(OriginClass::User, OriginClass::User)
            .rate_limit(OriginClass::User, 1000)
            .build();

        assert_eq!(policy.epoch_duration, 3600);
        assert!(policy.is_allowed(OriginClass::Genesis, OriginClass::User));
        assert_eq!(policy.get_rate_limit(OriginClass::User), 1000);
    }

    #[test]
    fn test_transitions_from() {
        let policy = OriginPolicy::default();
        let from_admin = policy.transitions_from(OriginClass::Admin);

        assert!(from_admin.contains(&OriginClass::User));
        assert!(from_admin.contains(&OriginClass::Admin));
        assert!(from_admin.contains(&OriginClass::Bridge));
    }

    #[test]
    fn test_serialization() {
        let policy = OriginPolicy::default();
        let json = serde_json::to_string(&policy).unwrap();
        let recovered: OriginPolicy = serde_json::from_str(&json).unwrap();

        assert_eq!(policy.compute_hash(), recovered.compute_hash());
    }
}
