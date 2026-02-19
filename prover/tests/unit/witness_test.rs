//! Unit tests for witness generation

use zk_origin::types::*;
use zk_origin::prover::WitnessGenerator;

fn create_generator() -> WitnessGenerator {
    let policy = OriginPolicy::default();
    let mut gen = WitnessGenerator::new(policy);
    gen.reset([0u8; 32]);
    gen
}

#[test]
fn test_witness_generator_init() {
    let gen = create_generator();
    
    assert_eq!(gen.current_depth(), 0);
    assert!(gen.current_lineage().is_genesis());
}

#[test]
fn test_generate_single_witness() {
    let mut gen = create_generator();
    
    let t = Transition::new(
        [0u8; 32],
        [1u8; 32],
        OriginClass::User,
        1000,
    );
    
    let witness = gen.generate_witness(&t).unwrap();
    
    assert_eq!(witness.new_origin, OriginClass::User);
    assert_eq!(witness.prev_origin, OriginClass::Genesis);
    assert_eq!(witness.prev_depth, 0);
    assert_eq!(gen.current_depth(), 1);
}

#[test]
fn test_witness_chain() {
    let mut gen = create_generator();
    
    for i in 0..10u8 {
        let t = Transition::new(
            [i; 32],
            [i + 1; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        
        let witness = gen.generate_witness(&t).unwrap();
        assert_eq!(witness.prev_depth, i as u64);
    }
    
    assert_eq!(gen.current_depth(), 10);
}

#[test]
fn test_witness_policy_proof_valid() {
    let mut gen = create_generator();
    
    let t = Transition::new(
        [0u8; 32],
        [1u8; 32],
        OriginClass::User,
        1000,
    );
    
    let witness = gen.generate_witness(&t).unwrap();
    
    // Policy proof should have correct depth
    assert_eq!(witness.policy_proof.len(), witness.policy_indices.len());
    assert!(!witness.policy_proof.is_empty());
}

#[test]
fn test_witness_counter_tracking() {
    let mut gen = create_generator();
    
    // Genesis -> Admin
    let t1 = Transition::new([0u8; 32], [1u8; 32], OriginClass::Admin, 1000);
    let w1 = gen.generate_witness(&t1).unwrap();
    
    // Check counters in witness
    assert_eq!(w1.prev_counters[OriginClass::Admin as usize], 0);
    
    // Admin -> Admin
    let t2 = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
    let w2 = gen.generate_witness(&t2).unwrap();
    
    // Previous counter should now be 1
    assert_eq!(w2.prev_counters[OriginClass::Admin as usize], 1);
}

#[test]
fn test_witness_epoch_change() {
    let mut policy = OriginPolicy::default();
    policy.epoch_duration = 100;
    
    let mut gen = WitnessGenerator::new(policy);
    gen.reset([0u8; 32]);
    
    // Transition at t=50 (epoch 0)
    let t1 = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 50);
    let w1 = gen.generate_witness(&t1).unwrap();
    assert_eq!(w1.epoch_id, 0);
    
    // Transition at t=150 (epoch 1)
    let t2 = Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 150);
    let w2 = gen.generate_witness(&t2).unwrap();
    assert_eq!(w2.epoch_id, 1);
}

#[test]
fn test_witness_commitment_computation() {
    let mut gen = create_generator();
    
    let t = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
    let witness = gen.generate_witness(&t).unwrap();
    
    // Compute expected values
    let transition_hash = witness.compute_transition_hash();
    let new_lineage = witness.compute_new_lineage_commitment();
    let new_counters = witness.compute_new_counter_commitment();
    
    // Verify they're non-zero
    assert_ne!(transition_hash, [0u8; 32]);
    assert_ne!(new_lineage, [0u8; 32]);
    assert_ne!(new_counters, [0u8; 32]);
}