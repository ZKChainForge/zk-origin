//! Simple example demonstrating basic ZK-ORIGIN usage

use zk_origin::{
    LineageProver, OriginPolicy, Transition, OriginClass,
    Result,
};

fn main() -> Result<()> {
    println!(" ZK-ORIGIN Simple Lineage \n");

    // Step 1: Create a policy
    println!("Step 1: Creating origin policy...");
    let policy = OriginPolicy::default();
    println!("  Policy created with {} allowed transitions", policy.num_allowed());
    println!("  Epoch duration: {} seconds", policy.epoch_duration);

    // Step 2: Create the prover
    println!("\nStep 2: Creating lineage prover...");
    let mut prover = LineageProver::new(policy)?;
    println!("  Prover created successfully");

    // Step 3: Initialize with genesis state
    println!("\nStep 3: Initializing with genesis state...");
    let genesis_hash = [0u8; 32]; // In practice, hash of actual genesis state
    prover.initialize(genesis_hash)?;
    println!("  Genesis commitment: {:?}", prover.current_lineage().unwrap());

    // Step 4: Add transitions
    println!("\nStep 4: Adding transitions...");
    
    let transitions = vec![
        ("Genesis → User", Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000)),
        ("User → User", Transition::new([1u8; 32], [2u8; 32], OriginClass::User, 2000)),
        ("User → User", Transition::new([2u8; 32], [3u8; 32], OriginClass::User, 3000)),
    ];

    for (desc, transition) in transitions {
        print!("  Adding {}: ", desc);
        match prover.add_transition(transition) {
            Ok(_) => println!(""),
            Err(e) => println!(" Error: {}", e),
        }
    }

    println!("  Current depth: {}", prover.current_depth());

    // Step 5: Generate proof
    println!("\nStep 5: Generating proof...");
    let proof = prover.finalize()?;
    
    println!("  Proof generated successfully!");
    println!("  - Lineage depth: {}", proof.num_steps);
    println!("  - Proof size: {} bytes", proof.proof_size());
    println!("  - Final lineage: {}", proof.final_lineage);

    // Step 6: Verify proof
    println!("\nStep 6: Verifying proof...");
    match proof.verify() {
        Ok(true) => println!("   Proof is valid!"),
        Ok(false) => println!("   Proof is invalid"),
        Err(e) => println!("   Verification error: {}", e),
    }

    Ok(())
}