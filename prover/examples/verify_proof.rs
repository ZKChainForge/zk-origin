//! Example demonstrating proof verification

use zk_origin::{
    LineageProver, LineageVerifier, OriginPolicy, Transition, OriginClass,
    Result,
    verifier::verify::verify_proof,
};

fn main() -> Result<()> {
    println!("ZK-ORIGIN Proof Verification \n");

    let genesis_hash = [42u8; 32];
    let policy = OriginPolicy::default();

    // Generate a proof
    println!("Step 1: Generating a proof...");
    let mut prover = LineageProver::new(policy.clone())?;
    prover.initialize(genesis_hash)?;

    for i in 0..5 {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(t)?;
    }

    let proof = prover.finalize()?;
    println!("  Proof generated: {} steps, {} bytes\n", proof.num_steps, proof.proof_size());

    // Method 1: Verify using LineageVerifier
    println!("Step 2: Verifying with LineageVerifier...");
    let verifier = LineageVerifier::new(genesis_hash, &policy);
    
    match verifier.verify(&proof) {
        Ok(true) => println!("   Proof is valid"),
        Ok(false) => println!("   Proof is invalid"),
        Err(e) => println!("   Error: {}", e),
    }

    // Get detailed verification results
    let detailed = verifier.verify_detailed(&proof);
    println!("  Detailed results: {}\n", detailed);

    // Method 2: Verify using standalone function
    println!("Step 3: Verifying with standalone function...");
    match verify_proof(&proof, genesis_hash, &policy) {
        Ok(true) => println!("   Proof is valid"),
        Ok(false) => println!("   Proof is invalid"),
        Err(e) => println!("   Error: {}", e),
    }

    // Method 3: Verify using proof's own method
    println!("\nStep 4: Verifying with proof.verify()...");
    match proof.verify() {
        Ok(true) => println!("   Proof is valid"),
        Ok(false) => println!("   Proof is invalid"),
        Err(e) => println!("   Error: {}", e),
    }

    // Test verification failure cases
    println!("\n Testing Verification Failures \n");

    // Wrong genesis
    println!("Test: Wrong genesis hash");
    let wrong_genesis = [0u8; 32];
    let verifier_wrong = LineageVerifier::new(wrong_genesis, &policy);
    match verifier_wrong.verify(&proof) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("   Correctly rejected: {}", e),
    }

    // Wrong policy
    println!("\nTest: Wrong policy");
    let wrong_policy = OriginPolicy::restrictive();
    let verifier_wrong_policy = LineageVerifier::new(genesis_hash, &wrong_policy);
    match verifier_wrong_policy.verify(&proof) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("   Correctly rejected: {}", e),
    }
    
    // Serialization round-trip

    println!("\n Testing Proof Serialization \n");
    
    let json = proof.to_json()?;
    println!("  JSON size: {} bytes", json.len());
    
    let recovered = zk_origin::LineageProof::from_json(&json)?;
    println!("  Recovered proof: {} steps", recovered.num_steps);
    
    match verifier.verify(&recovered) {
        Ok(true) => println!("   Recovered proof is valid"),
        Ok(false) => println!("   Recovered proof is invalid"),
        Err(e) => println!("   Error: {}", e),
    }

    Ok(())
}