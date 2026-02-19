//! Integration tests for rate limiting

use zk_origin::{
    LineageProver, OriginPolicy, Transition, OriginClass,
    ZkOriginError,
};

#[test]
fn test_admin_rate_limit() {
    let policy = OriginPolicy::default(); // Admin limit = 10
    
    let mut prover = LineageProver::new(policy.clone()).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    // Genesis -> Admin
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::Admin, 1000
    )).unwrap();
    
    // 9 more Admin -> Admin (total 10)
    for i in 1..10u8 {
        prover.add_transition(Transition::new(
            [i; 32], [i + 1; 32], OriginClass::Admin, (i as u64 + 1) * 1000
        )).unwrap();
    }
    
    // 11th should fail
    let result = prover.add_transition(Transition::new(
        [10u8; 32], [11u8; 32], OriginClass::Admin, 11000
    ));
    
    assert!(matches!(result, Err(ZkOriginError::RateLimitExceeded { .. })));
}

#[test]
fn test_user_unlimited() {
    let policy = OriginPolicy::default(); // User limit = MAX
    
    let mut prover = LineageProver::new(policy).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    // Should be able to add many user transitions
    for i in 0..1000u16 {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            i as u64 * 1000,
        );
        prover.add_transition(t).unwrap();
    }
    
    assert_eq!(prover.current_depth(), 1000);
}

#[test]
fn test_epoch_reset() {
    let mut policy = OriginPolicy::default();
    policy.epoch_duration = 100; // Short epochs for testing
    policy.set_rate_limit(OriginClass::Admin, 3);
    
    let mut prover = LineageProver::new(policy).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    // Epoch 0: 3 admin transitions
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::Admin, 10
    )).unwrap();
    prover.add_transition(Transition::new(
        [1u8; 32], [2u8; 32], OriginClass::Admin, 20
    )).unwrap();
    prover.add_transition(Transition::new(
        [2u8; 32], [3u8; 32], OriginClass::Admin, 30
    )).unwrap();
    
    // 4th should fail in epoch 0
    assert!(prover.add_transition(Transition::new(
        [3u8; 32], [4u8; 32], OriginClass::Admin, 40
    )).is_err());
    
    // Move to epoch 1 (t >= 100)
    prover.add_transition(Transition::new(
        [3u8; 32], [4u8; 32], OriginClass::Admin, 100
    )).unwrap();
    
    // Should be able to do 2 more in epoch 1
    prover.add_transition(Transition::new(
        [4u8; 32], [5u8; 32], OriginClass::Admin, 110
    )).unwrap();
}

#[test]
fn test_different_class_limits() {
    let mut policy = OriginPolicy::default();
    policy.set_rate_limit(OriginClass::Admin, 2);
    policy.set_rate_limit(OriginClass::System, 3);
    
    // Add System -> User as allowed transition for this test
    policy.allow(OriginClass::System, OriginClass::Admin);
    
    let mut prover = LineageProver::new(policy).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    // Use up admin limit
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::Admin, 1000
    )).unwrap();
    prover.add_transition(Transition::new(
        [1u8; 32], [2u8; 32], OriginClass::Admin, 2000
    )).unwrap();
    
    // Admin limit reached
    assert!(prover.add_transition(Transition::new(
        [2u8; 32], [3u8; 32], OriginClass::Admin, 3000
    )).is_err());
    
    // But System should still have quota (via Admin -> System)
    prover.add_transition(Transition::new(
        [2u8; 32], [3u8; 32], OriginClass::System, 3000
    )).unwrap();
}