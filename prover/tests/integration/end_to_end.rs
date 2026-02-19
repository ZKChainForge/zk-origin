//! End-to-end integration tests

use zk_origin::{
    LineageProver, OriginPolicy, Transition, OriginClass,
    LineageVerifier, LineageProof,
    utils::serialize::{save_json, load_json},
};
use tempfile::tempdir;

#[test]
fn test_full_workflow() {
    // 1. Setup
    let genesis = [0u8; 32];
    let policy = OriginPolicy::default();
    
    // 2. Create prover and initialize
    let mut prover = LineageProver::new(policy.clone()).unwrap();
    prover.initialize(genesis).unwrap();
    
    // 3. Add transitions
    let transitions = vec![
        Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000),
        Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 2000),
        Transition::new([2u8; 32], [3u8; 32], OriginClass::User, 3000),
    ];
    
    for t in transitions {
        prover.add_transition(t).unwrap();
    }
    
    // 4. Generate proof
    let proof = prover.finalize().unwrap();
    
    // 5. Serialize proof
    let json = proof.to_json().unwrap();
    
    // 6. Deserialize proof
    let recovered: LineageProof = LineageProof::from_json(&json).unwrap();
    
    // 7. Verify proof
    let verifier = LineageVerifier::new(genesis, &policy);
    assert!(verifier.verify(&recovered).unwrap());
}

#[test]
fn test_proof_file_persistence() {
    let dir = tempdir().unwrap();
    let proof_path = dir.path().join("proof.json");
    
    // Generate proof
    let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    prover.add_transition(Transition::new(
        [0u8; 32], [1u8; 32], OriginClass::User, 1000
    )).unwrap();
    
    let proof = prover.finalize().unwrap();
    
    // Save to file
    save_json(&proof, &proof_path).unwrap();
    
    // Load from file
    let loaded: LineageProof = load_json(&proof_path).unwrap();
    
    // Verify loaded proof
    assert!(loaded.verify().unwrap());
    assert_eq!(proof.num_steps, loaded.num_steps);
}

#[test]
fn test_multiple_provers_same_policy() {
    let policy = OriginPolicy::default();
    let genesis = [0u8; 32];
    
    // Create two provers
    let mut prover1 = LineageProver::new(policy.clone()).unwrap();
    let mut prover2 = LineageProver::new(policy.clone()).unwrap();
    
    prover1.initialize(genesis).unwrap();
    prover2.initialize(genesis).unwrap();
    
    // Same transitions
    let t = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
    
    prover1.add_transition(t.clone()).unwrap();
    prover2.add_transition(t).unwrap();
    
    let proof1 = prover1.finalize().unwrap();
    let proof2 = prover2.finalize().unwrap();
    
    // Both should be valid
    let verifier = LineageVerifier::new(genesis, &policy);
    assert!(verifier.verify(&proof1).unwrap());
    assert!(verifier.verify(&proof2).unwrap());
    
    // Same lineage commitment
    assert_eq!(
        proof1.final_lineage.value,
        proof2.final_lineage.value
    );
}

#[test]
fn test_verifier_reuse() {
    let policy = OriginPolicy::default();
    let genesis = [0u8; 32];
    let verifier = LineageVerifier::new(genesis, &policy);
    
    // Generate multiple proofs
    for num_steps in [1, 5, 10] {
        let mut prover = LineageProver::new(policy.clone()).unwrap();
        prover.initialize(genesis).unwrap();
        
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
        
        // Same verifier can verify all proofs
        assert!(verifier.verify(&proof).unwrap());
    }
}

#[test]
fn test_proof_summary_display() {
    let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    for i in 0..5 {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(t).unwrap();
    }
    
    let proof = prover.finalize().unwrap();
    let summary = proof.summary();
    
    println!("{}", summary);
    
    assert!(summary.to_string().contains("Lineage"));
    assert!(summary.to_string().contains("5"));
}