//! Nova demo - only works with real-nova feature
//!
//! Run with:
//! cargo run --example nova_demo --features real-nova --no-default-features

#[cfg(feature = "real-nova")]
fn main() {
    use zk_origin::*;
    use zk_origin::prover::nova_prover::{NovaParams, NovaLineageProver};
    use std::time::Instant;

    println!("ZK-ORIGIN Nova Demo");
    println!("====================\n");

    // Setup
    println!("Setting up Nova parameters (this takes 30-120 seconds)...");
    let start = Instant::now();
    
    let policy = OriginPolicy::default();
    let params = NovaParams::setup(policy.compute_hash())
        .expect("Failed to setup Nova");
    
    println!("Setup completed in {:?}\n", start.elapsed());

    // Create prover
    let mut prover = NovaLineageProver::new(&params);
    
    // Initialize
    let hasher = zk_origin::hash::poseidon_native::NativePoseidonHasher::new();
    let genesis_state = [0u8; 32];
    let genesis_lineage = hasher.compute_genesis_commitment(&genesis_state);
    let initial_counters = hasher.compute_counter_commitment(0, &[0; 6]);
    
    prover.initialize(genesis_lineage, initial_counters)
        .expect("Failed to initialize");

    // Create witness generator
    let mut witness_gen = WitnessGenerator::new(policy);
    witness_gen.reset(genesis_state);

    // Add transitions
    println!("Adding transitions...");
    for i in 0..3 {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        
        let witness = witness_gen.generate_witness(&transition)
            .expect("Failed to generate witness");
        
        let start = Instant::now();
        prover.prove_step(&witness)
            .expect("Failed to prove step");
        println!("  Step {} proved in {:?}", i + 1, start.elapsed());
    }

    // Finalize
    println!("\nFinalizing proof...");
    let start = Instant::now();
    let proof = prover.finalize().expect("Failed to finalize");
    println!("Finalized in {:?}", start.elapsed());

    // Summary
    println!("\n=== PROOF SUMMARY ===");
    println!("Depth: {} steps", proof.num_steps);
    println!("Proof size: {} bytes ({:.2} KB)", proof.proof_size(), proof.proof_size() as f64 / 1024.0);
    println!("Is real ZK: {}", proof.is_real_zk());
}

#[cfg(not(feature = "real-nova"))]
fn main() {
    println!("This example requires the 'real-nova' feature.");
    println!("Run with:");
    println!("  cargo run --example nova_demo --features real-nova --no-default-features");
}