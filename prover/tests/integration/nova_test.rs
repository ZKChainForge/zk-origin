//! Integration tests for Nova proving
//!
//! These tests are expensive and marked as ignored by default.
//! Run with: cargo test --release nova -- --ignored

use zk_origin::{
    OriginPolicy, OriginClass, Transition,
    prover::{NovaParams, NovaLineageProver, WitnessGenerator},
    Result,
};

#[test]
#[ignore = "expensive: Nova setup takes ~30 seconds"]
fn test_nova_setup() -> Result<()> {
    let policy_root = [0u8; 32];
    let params = NovaParams::setup(policy_root)?;
    
    assert!(params.setup_time_ms > 0);
    Ok(())
}

#[test]
#[ignore = "expensive: Nova proving takes several seconds"]
fn test_nova_single_step() -> Result<()> {
    let policy = OriginPolicy::default();
    let policy_root = policy.compute_hash();
    
    let params = NovaParams::setup(policy_root)?;
    let mut prover = NovaLineageProver::new(params);
    prover.initialize([0u8; 32], 0)?;
    
    let mut witness_gen = WitnessGenerator::new(policy);
    witness_gen.reset([0u8; 32]);
    
    let transition = Transition::new(
        [0u8; 32],
        [1u8; 32],
        OriginClass::User,
        1000,
    );
    
    let witness = witness_gen.generate_witness(&transition)?;
    prover.prove_step(&witness)?;
    
    assert!(prover.verify()?);
    assert_eq!(prover.current_depth(), 1);
    
    Ok(())
}

#[test]
#[ignore = "expensive: Nova proving takes several seconds"]
fn test_nova_multiple_steps() -> Result<()> {
    let policy = OriginPolicy::default();
    let policy_root = policy.compute_hash();
    
    let params = NovaParams::setup(policy_root)?;
    let mut prover = NovaLineageProver::new(params);
    prover.initialize([0u8; 32], 0)?;
    
    let mut witness_gen = WitnessGenerator::new(policy);
    witness_gen.reset([0u8; 32]);
    
    for i in 0..3 {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        
        let witness = witness_gen.generate_witness(&transition)?;
        prover.prove_step(&witness)?;
    }
    
    assert!(prover.verify()?);
    assert_eq!(prover.current_depth(), 3);
    
    Ok(())
}

#[test]
#[ignore = "very expensive: Nova compression takes 30+ seconds"]
fn test_nova_compress() -> Result<()> {
    let policy = OriginPolicy::default();
    let policy_root = policy.compute_hash();
    
    let params = NovaParams::setup(policy_root)?;
    let mut prover = NovaLineageProver::new(params);
    prover.initialize([0u8; 32], 0)?;
    
    let mut witness_gen = WitnessGenerator::new(policy);
    witness_gen.reset([0u8; 32]);
    
    // Add a few steps
    for i in 0..2 {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        
        let witness = witness_gen.generate_witness(&transition)?;
        prover.prove_step(&witness)?;
    }
    
    // Compress
    let compressed = prover.compress()?;
    
    // Check properties
    assert_eq!(compressed.num_steps, 2);
    assert!(compressed.to_bytes()?.len() > 1000); // Should be several KB
    
    Ok(())
}

#[test]
#[ignore = "very expensive: Full Nova flow"]
fn test_nova_full_flow() -> Result<()> {
    let policy = OriginPolicy::default();
    let policy_root = policy.compute_hash();
    
    // Setup
    let params = NovaParams::setup(policy_root)?;
    
    // Initialize
    let mut prover = NovaLineageProver::new(params);
    prover.initialize([0u8; 32], 0)?;
    
    let mut witness_gen = WitnessGenerator::new(policy);
    witness_gen.reset([0u8; 32]);
    
    // Prove steps
    let transitions = vec![
        (OriginClass::User, "Genesis → User"),
        (OriginClass::User, "User → User"),
    ];
    
    for (i, (origin, _)) in transitions.iter().enumerate() {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            *origin,
            (i as u64 + 1) * 1000,
        );
        
        let witness = witness_gen.generate_witness(&transition)?;
        prover.prove_step(&witness)?;
    }
    
    // Verify intermediate
    assert!(prover.verify()?);
    
    // Finalize
    let proof = prover.finalize()?;
    
    // Check proof
    assert_eq!(proof.num_steps, 2);
    assert!(proof.proof_size() > 1000);
    
    Ok(())
}