use zk_origin_prover::{LineageProver, Origin, Transition};
use zk_origin_prover::prover::ProverError;

#[test]
fn test_user_to_admin_rejected() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    // Genesis -> User
    prover.add_transition(Transition::new(
        genesis_state,
        [1u8; 32],
        Origin::User,
        1000,
    )).unwrap();
    
    // User -> Admin (should fail)
    let result = prover.add_transition(Transition::new(
        [1u8; 32],
        [2u8; 32],
        Origin::Admin,
        2000,
    ));
    
    assert!(matches!(result, Err(ProverError::PolicyViolation { .. })));
}

#[test]
fn test_user_to_bridge_rejected() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    // Genesis -> User
    prover.add_transition(Transition::new(
        genesis_state,
        [1u8; 32],
        Origin::User,
        1000,
    )).unwrap();
    
    // User -> Bridge (should fail)
    let result = prover.add_transition(Transition::new(
        [1u8; 32],
        [2u8; 32],
        Origin::Bridge,
        2000,
    ));
    
    assert!(matches!(result, Err(ProverError::PolicyViolation { .. })));
}

#[test]
fn test_bridge_to_admin_rejected() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    // Genesis -> Admin -> Bridge
    prover.add_transition(Transition::new(
        genesis_state,
        [1u8; 32],
        Origin::Admin,
        1000,
    )).unwrap();
    
    prover.add_transition(Transition::new(
        [1u8; 32],
        [2u8; 32],
        Origin::Bridge,
        2000,
    )).unwrap();
    
    // Bridge -> Admin (should fail)
    let result = prover.add_transition(Transition::new(
        [2u8; 32],
        [3u8; 32],
        Origin::Admin,
        3000,
    ));
    
    assert!(matches!(result, Err(ProverError::PolicyViolation { .. })));
}

#[test]
fn test_return_to_genesis_rejected() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    // Genesis -> User
    prover.add_transition(Transition::new(
        genesis_state,
        [1u8; 32],
        Origin::User,
        1000,
    )).unwrap();
    
    // User -> Genesis (should fail)
    let result = prover.add_transition(Transition::new(
        [1u8; 32],
        [2u8; 32],
        Origin::Genesis,
        2000,
    ));
    
    assert!(matches!(result, Err(ProverError::PolicyViolation { .. })));
}

#[test]
fn test_bridge_chaining_rejected() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    // Genesis -> Admin -> Bridge
    prover.add_transition(Transition::new(
        genesis_state,
        [1u8; 32],
        Origin::Admin,
        1000,
    )).unwrap();
    
    prover.add_transition(Transition::new(
        [1u8; 32],
        [2u8; 32],
        Origin::Bridge,
        2000,
    )).unwrap();
    
    // Bridge -> Bridge (should fail)
    let result = prover.add_transition(Transition::new(
        [2u8; 32],
        [3u8; 32],
        Origin::Bridge,
        3000,
    ));
    
    assert!(matches!(result, Err(ProverError::PolicyViolation { .. })));
}