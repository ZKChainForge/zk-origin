//! Policy tree management

use crate::types::OriginClass;
use poseidon_rs::{Poseidon, Fr, FrRepr};
use ff_ce::PrimeField;  // ← Use ff_ce instead of ff
use std::collections::HashMap;

pub mod tree;

pub use tree::{PolicyTree, PolicyProof};

/// Get default policy matrix
pub fn default_policy_matrix() -> HashMap<(OriginClass, OriginClass), bool> {
    let mut policy = HashMap::new();
    
    use OriginClass::*;
    
    // Genesis → User, Admin, System
    policy.insert((Genesis, User), true);
    policy.insert((Genesis, Admin), true);
    policy.insert((Genesis, System), true);
    
    // User → User
    policy.insert((User, User), true);
    
    // Admin → User, Admin, Bridge, System
    policy.insert((Admin, User), true);
    policy.insert((Admin, Admin), true);
    policy.insert((Admin, Bridge), true);
    policy.insert((Admin, System), true);
    
    // Bridge → User
    policy.insert((Bridge, User), true);
    
    // Governance → ALL
    for to_class in OriginClass::all() {
        policy.insert((Governance, to_class), true);
    }
    
    // System → User, System
    policy.insert((System, User), true);
    policy.insert((System, System), true);
    
    // Emergency → User, Admin, System
    policy.insert((Emergency, User), true);
    policy.insert((Emergency, Admin), true);
    policy.insert((Emergency, System), true);
    
    policy
}

/// Get all allowed transitions as leaves
pub fn get_policy_leaves() -> Vec<(OriginClass, OriginClass)> {
    let policy = default_policy_matrix();
    policy.keys().cloned().collect()
}

/// Hash a transition for policy tree
pub fn hash_transition(from: OriginClass, to: OriginClass) -> Fr {
    let poseidon = Poseidon::new();
    
    // Create Fr from u64 using from_repr
    let from_repr = FrRepr::from(from as u64);
    let to_repr = FrRepr::from(to as u64);
    
    let from_fr = Fr::from_repr(from_repr).unwrap();
    let to_fr = Fr::from_repr(to_repr).unwrap();
    
    let inputs = vec![from_fr, to_fr];
    poseidon.hash(inputs).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_policy() {
        let policy = default_policy_matrix();
        
        // Test some allowed transitions
        assert!(policy.get(&(OriginClass::Genesis, OriginClass::User)).copied().unwrap_or(false));
        assert!(policy.get(&(OriginClass::User, OriginClass::User)).copied().unwrap_or(false));
        assert!(policy.get(&(OriginClass::Admin, OriginClass::Bridge)).copied().unwrap_or(false));
        
        // Test some forbidden transitions
        assert!(!policy.get(&(OriginClass::User, OriginClass::Admin)).copied().unwrap_or(false));
        assert!(!policy.get(&(OriginClass::Bridge, OriginClass::Admin)).copied().unwrap_or(false));
    }
    
    #[test]
    fn test_hash_transition() {
        let hash1 = hash_transition(OriginClass::User, OriginClass::User);
        let hash2 = hash_transition(OriginClass::User, OriginClass::Admin);
        
        assert_ne!(hash1, hash2);
    }
}