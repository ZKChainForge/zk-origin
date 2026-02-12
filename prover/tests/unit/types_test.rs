use zk_origin_prover::types::{Origin, Policy, Transition, LineageCommitment};

#[test]
fn test_origin_conversion() {
    assert_eq!(Origin::Genesis.as_u8(), 0);
    assert_eq!(Origin::User.as_u8(), 1);
    assert_eq!(Origin::Admin.as_u8(), 2);
    assert_eq!(Origin::Bridge.as_u8(), 3);
    
    assert_eq!(Origin::from_u8(0), Some(Origin::Genesis));
    assert_eq!(Origin::from_u8(1), Some(Origin::User));
    assert_eq!(Origin::from_u8(2), Some(Origin::Admin));
    assert_eq!(Origin::from_u8(3), Some(Origin::Bridge));
    assert_eq!(Origin::from_u8(4), None);
}

#[test]
fn test_policy_default() {
    let policy = Policy::default_policy();
    
    // Allowed
    assert!(policy.is_allowed(Origin::Genesis, Origin::User, 0));
    assert!(policy.is_allowed(Origin::Genesis, Origin::Admin, 0));
    assert!(policy.is_allowed(Origin::User, Origin::User, 1));
    assert!(policy.is_allowed(Origin::Admin, Origin::User, 1));
    assert!(policy.is_allowed(Origin::Admin, Origin::Admin, 1));
    assert!(policy.is_allowed(Origin::Admin, Origin::Bridge, 1));
    assert!(policy.is_allowed(Origin::Bridge, Origin::User, 1));
    
    // Not allowed
    assert!(!policy.is_allowed(Origin::User, Origin::Admin, 1));
    assert!(!policy.is_allowed(Origin::User, Origin::Bridge, 1));
    assert!(!policy.is_allowed(Origin::Bridge, Origin::Admin, 1));
    assert!(!policy.is_allowed(Origin::Bridge, Origin::Bridge, 1));
    
    // Cannot return to genesis
    assert!(!policy.is_allowed(Origin::User, Origin::Genesis, 5));
}

#[test]
fn test_lineage_commitment() {
    let genesis = LineageCommitment::genesis([0u8; 32]);
    assert_eq!(genesis.depth, 0);
    
    let next = LineageCommitment::new([1u8; 32], 1);
    assert_eq!(next.depth, 1);
    
    assert_eq!(genesis.to_hex().len(), 64);
}

#[test]
fn test_transition() {
    let t = Transition::new(
        [0u8; 32],
        [1u8; 32],
        Origin::User,
        1000,
    );
    
    assert_eq!(t.origin, Origin::User);
    assert_eq!(t.timestamp, 1000);
}