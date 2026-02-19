//! Unit tests for circuit components

use zk_origin::circuit::step::{LineageStepCircuit, NUM_ORIGIN_CLASSES, POLICY_TREE_DEPTH};
use bellpepper_core::test_cs::TestConstraintSystem;
use bellpepper_core::num::AllocatedNum;
use pasta_curves::Fp;

fn create_test_circuit() -> LineageStepCircuit<Fp> {
    LineageStepCircuit::new(
        Fp::from(1u64),
        Fp::from(2u64),
        0, // Genesis
        1, // User
        1000,
        0,
        Fp::from(100u64),
        vec![Fp::from(1u64); POLICY_TREE_DEPTH],
        vec![false; POLICY_TREE_DEPTH],
        0,
        [0; NUM_ORIGIN_CLASSES],
        [1, u32::MAX, 10, 100, 5, 1000],
    )
}

#[test]
fn test_circuit_default() {
    let circuit: LineageStepCircuit<Fp> = LineageStepCircuit::default();
    assert!(circuit.prev_state_hash.is_none());
}

#[test]
fn test_circuit_synthesis_succeeds() {
    let mut cs = TestConstraintSystem::<Fp>::new();
    let circuit = create_test_circuit();
    
    let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
    let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
    
    let result = circuit.synthesize_step(&mut cs, &[z0, z1]);
    
    assert!(result.is_ok());
}

#[test]
fn test_circuit_constraints_satisfied() {
    let mut cs = TestConstraintSystem::<Fp>::new();
    let circuit = create_test_circuit();
    
    let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
    let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
    
    let _ = circuit.synthesize_step(&mut cs, &[z0, z1]).unwrap();
    
    assert!(cs.is_satisfied(), "Circuit should be satisfied");
}

#[test]
fn test_circuit_output_count() {
    let mut cs = TestConstraintSystem::<Fp>::new();
    let circuit = create_test_circuit();
    
    let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
    let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
    
    let z_prime = circuit.synthesize_step(&mut cs, &[z0, z1]).unwrap();
    
    assert_eq!(z_prime.len(), 2, "Should output 2 state elements");
}

#[test]
fn test_circuit_constraint_count_reasonable() {
    let mut cs = TestConstraintSystem::<Fp>::new();
    let circuit = create_test_circuit();
    
    let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
    let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
    
    let _ = circuit.synthesize_step(&mut cs, &[z0, z1]).unwrap();
    
    let num_constraints = cs.num_constraints();
    println!("Circuit has {} constraints", num_constraints);
    
    // Sanity check: should be less than 100k for our simple circuit
    assert!(num_constraints < 100_000, "Too many constraints: {}", num_constraints);
    assert!(num_constraints > 0, "Should have some constraints");
}

#[test]
fn test_circuit_different_origins() {
    for prev_origin in 0..6u64 {
        for new_origin in 0..6u64 {
            let mut cs = TestConstraintSystem::<Fp>::new();
            
            let circuit = LineageStepCircuit::new(
                Fp::from(1u64),
                Fp::from(2u64),
                prev_origin,
                new_origin,
                1000,
                5,
                Fp::from(100u64),
                vec![Fp::from(1u64); POLICY_TREE_DEPTH],
                vec![false; POLICY_TREE_DEPTH],
                0,
                [0; NUM_ORIGIN_CLASSES],
                [1, u32::MAX, 10, 100, 5, 1000],
            );
            
            let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
            let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
            
            let result = circuit.synthesize_step(&mut cs, &[z0, z1]);
            
            assert!(result.is_ok(), "Failed for origins ({}, {})", prev_origin, new_origin);
        }
    }
}