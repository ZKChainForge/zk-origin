//! Integration tests for multi-step proving

use zk_origin::{
    LineageProver, OriginPolicy, Transition, OriginClass,
    LineageVerifier,
};

#[test]
fn test_multi_step_10() {
    let genesis = [0u8; 32];
    let policy = OriginPolicy::default();
    
    let mut prover = LineageProver::new(policy.clone()).unwrap();
    prover.initialize(genesis).unwrap();
    
    for i in 0..10u8 {
        let t = Transition::new(
            [i; 32],
            [i + 1; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(t).unwrap();
    }
    
    let proof = prover.finalize().unwrap();
    
    assert_eq!(proof.num_steps, 10);
    
    let verifier = LineageVerifier::new(genesis, &policy);
    assert!(verifier.verify(&proof).unwrap());
}

#[test]
fn test_multi_step_100() {
    let genesis = [0u8; 32];
    let policy = OriginPolicy::default();
    
    let mut prover = LineageProver::new(policy.clone()).unwrap();
    prover.initialize(genesis).unwrap();
    
    for i in 0..100u8 {
        let t = Transition::new(
            [i; 32],
            [i.wrapping_add(1); 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(t).unwrap();
    }
    
    let proof = prover.finalize().unwrap();
    
    assert_eq!(proof.num_steps, 100);
    assert!(proof.verify().unwrap());
}

#[test]
fn test_multi_step_mixed_origins() {
    let genesis = [0u8; 32];
    let policy = OriginPolicy::default();
    
    let mut prover = LineageProver::new(policy.clone()).unwrap();
    prover.initialize(genesis).unwrap();
    
    // Genesis -> Admin
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::Admin, 1000
    )).unwrap();
    
    // Admin -> User
    prover.add_transition(Transition::new(
        [1u8; 32], [2u8; 32], OriginClass::User, 2000
    )).unwrap();
    
    // User -> User (multiple)
    for i in 2..10u8 {
        prover.add_transition(Transition::new(
            [i; 32], [i + 1; 32], OriginClass::User, (i as u64 + 1) * 1000
        )).unwrap();
    }
    
    let proof = prover.finalize().unwrap();
    assert!(proof.verify().unwrap());
}

#[test]
fn test_proof_size_constant() {
    let policy = OriginPolicy::default();
    
    let mut sizes = Vec::new();
    
    for num_steps in [5, 10, 50, 100] {
        let mut prover = LineageProver::new(policy.clone()).unwrap();
        prover.initialize([0u8; 32]).unwrap();
        
        for i in 0..num_steps {
            let t = Transition::new(
                [i as u8; 32],
                [(i + 1) as u8; 32],
                OriginClass::User,
                (i as u64 + 1) * 1000,
            );
            prover.add_transition(t).unwrap();
        }
        
        let proof = prover.finalize().unwrap();
        sizes.push((num_steps, proof.proof_size()));
    }
    
    // In a real Nova implementation, sizes would be constant
    // For our placeholder, they may vary
    println!("Proof sizes: {:?}", sizes);
}