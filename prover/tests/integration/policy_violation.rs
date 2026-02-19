//! Integration tests for policy violation detection

use zk_origin::{
    LineageProver, OriginPolicy, Transition, OriginClass,
    ZkOriginError,
};

#[test]
fn test_user_to_admin_blocked() {
    let policy = OriginPolicy::default();
    
    let mut prover = LineageProver::new(policy).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    // Genesis -> User (valid)
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::User, 1000
    )).unwrap();
    
    // User -> Admin (invalid)
    let result = prover.add_transition(Transition::new(
        [1u8; 32], [2u8; 32], OriginClass::Admin, 2000
    ));
    
    assert!(matches!(result, Err(ZkOriginError::PolicyViolation { .. })));
}

#[test]
fn test_bridge_to_admin_blocked() {
    let policy = OriginPolicy::default();
    
    let mut prover = LineageProver::new(policy).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    // Genesis -> Admin (valid)
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::Admin, 1000
    )).unwrap();
    
    // Admin -> Bridge (valid)
    prover.add_transition(Transition::new(
        [1u8; 32], [2u8; 32], OriginClass::Bridge, 2000
    )).unwrap();
    
    // Bridge -> Admin (invalid)
    let result = prover.add_transition(Transition::new(
        [2u8; 32], [3u8; 32], OriginClass::Admin, 3000
    ));
    
    assert!(matches!(result, Err(ZkOriginError::PolicyViolation { .. })));
}

#[test]
fn test_restrictive_policy() {
    let policy = OriginPolicy::restrictive();
    
    let mut prover = LineageProver::new(policy).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    // Genesis -> User (valid)
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::User, 1000
    )).unwrap();
    
    // User -> Admin (invalid in restrictive policy)
    let result = prover.add_transition(Transition::new(
        [1u8; 32], [2u8; 32], OriginClass::Admin, 2000
    ));
    
    assert!(result.is_err());
}

#[test]
fn test_state_preserved_after_violation() {
    let policy = OriginPolicy::default();
    
    let mut prover = LineageProver::new(policy).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    // Genesis -> User (valid)
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::User, 1000
    )).unwrap();
    
    let depth_before = prover.current_depth();
    
    // User -> Admin (invalid)
    let _ = prover.add_transition(Transition::new(
        [1u8; 32], [2u8; 32], OriginClass::Admin, 2000
    ));
    
    // Depth should be unchanged
    assert_eq!(prover.current_depth(), depth_before);
    
    // Can still add valid transition
    prover.add_transition(Transition::new(
        [1u8; 32], [2u8; 32], OriginClass::User, 2000
    )).unwrap();
    
    assert_eq!(prover.current_depth(), depth_before + 1);
}