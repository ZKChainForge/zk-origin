//! Integration test for single step proving

use zk_origin::{
    LineageProver, OriginPolicy, Transition, OriginClass,
    LineageVerifier,
};

#[test]
fn test_single_step_prove_verify() {
    let genesis = [0u8; 32];
    let policy = OriginPolicy::default();
    
    // Create and initialize prover
    let mut prover = LineageProver::new(policy.clone()).unwrap();
    prover.initialize(genesis).unwrap();
    
    // Add single transition
    let t = Transition::new(
        genesis,
        [1u8; 32],
        OriginClass::User,
        1000,
    );
    prover.add_transition(t).unwrap();
    
    // Generate proof
    let proof = prover.finalize().unwrap();
    
    // Verify
    let verifier = LineageVerifier::new(genesis, &policy);
    assert!(verifier.verify(&proof).unwrap());
}

#[test]
fn test_single_step_different_origins() {
    let genesis = [0u8; 32];
    let policy = OriginPolicy::default();
    
    // Test each valid origin from genesis
    for origin in [OriginClass::User, OriginClass::Admin, OriginClass::System] {
        if policy.is_allowed(OriginClass::Genesis, origin) {
            let mut prover = LineageProver::new(policy.clone()).unwrap();
            prover.initialize(genesis).unwrap();
            
            let t = Transition::new(genesis, [1u8; 32], origin, 1000);
            prover.add_transition(t).unwrap();
            
            let proof = prover.finalize().unwrap();
            assert!(proof.verify().unwrap(), "Failed for origin {:?}", origin);
        }
    }
}

#[test]
fn test_single_step_proof_metadata() {
    let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    let t = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
    prover.add_transition(t).unwrap();
    
    let proof = prover.finalize().unwrap();
    
    // Check metadata
    assert!(proof.metadata.generated_at > 0);
    assert!(!proof.metadata.prover_version.is_empty());
}