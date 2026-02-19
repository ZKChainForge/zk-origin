//! Unit tests for type definitions

use zk_origin::types::*;

#[test]
fn test_origin_class_all_values() {
    let all = OriginClass::all();
    assert_eq!(all.len(), 6);
    
    assert!(all.contains(&OriginClass::Genesis));
    assert!(all.contains(&OriginClass::User));
    assert!(all.contains(&OriginClass::Admin));
    assert!(all.contains(&OriginClass::Bridge));
    assert!(all.contains(&OriginClass::Governance));
    assert!(all.contains(&OriginClass::System));
}

#[test]
fn test_origin_class_field_element() {
    assert_eq!(OriginClass::Genesis.to_field_element(), 0);
    assert_eq!(OriginClass::User.to_field_element(), 1);
    assert_eq!(OriginClass::System.to_field_element(), 5);
}

#[test]
fn test_lineage_commitment_display() {
    let commitment = LineageCommitment::new([0xAB; 32], 42);
    let display = format!("{}", commitment);
    
    assert!(display.contains("@42"));
    assert!(display.contains("abab"));
}

#[test]
fn test_transition_chain_validation() {
    let mut seq = TransitionSequence::new();
    
    // Build a valid chain
    for i in 0..5u8 {
        let t = Transition::new(
            [i; 32],
            [i + 1; 32],
            OriginClass::User,
            i as u64 * 1000,
        );
        seq.push(t);
    }
    
    assert!(seq.validate_chain());
    assert_eq!(seq.len(), 5);
}

#[test]
fn test_policy_builder_chain() {
    let policy = PolicyBuilder::new()
        .epoch_duration(7200)
        .allow(OriginClass::Genesis, OriginClass::User)
        .allow(OriginClass::User, OriginClass::User)
        .rate_limit(OriginClass::User, 500)
        .build();
    
    assert_eq!(policy.epoch_duration, 7200);
    assert!(policy.is_allowed(OriginClass::Genesis, OriginClass::User));
    assert!(policy.is_allowed(OriginClass::User, OriginClass::User));
    assert!(!policy.is_allowed(OriginClass::User, OriginClass::Admin));
    assert_eq!(policy.get_rate_limit(OriginClass::User), 500);
}

#[test]
fn test_epoch_counters_multiple_increments() {
    let mut counters = EpochCounters::new(0);
    
    for _ in 0..100 {
        counters.increment(OriginClass::User);
    }
    
    assert_eq!(counters.get(OriginClass::User), 100);
    assert_eq!(counters.get(OriginClass::Admin), 0);
}

#[test]
fn test_proof_summary() {
    let proof = LineageProof::new(
        vec![1, 2, 3],
        LineageCommitment::new([1u8; 32], 10),
        CounterCommitment::new([2u8; 32], 0),
        LineageCommitment::genesis([0u8; 32]),
        10,
        [3u8; 32],
    );
    
    let summary = proof.summary();
    assert_eq!(summary.depth, 10);
    assert_eq!(summary.proof_size, 3);
}