//! Nova ZK proving demonstration
//!
//! This example shows how to use real Nova IVC proofs.
//! 
//! WARNING: This is SLOW! Expected times:
//! - Setup: ~15-30 seconds
//! - Per step: ~100-500ms
//! - Compression: ~10-60 seconds
//!
//! Run with: cargo run --release --example nova_demo

use zk_origin::{
    OriginPolicy, OriginClass, Transition,
    prover::{NovaParams, NovaLineageProver, WitnessGenerator},
    Result,
};
use std::time::Instant;

fn main() -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                   NOVA ZK PROVING DEMO                        ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    // ========================================
    // Step 1: Setup (SLOW: 15-30 seconds)
    // ========================================
    println!(" Step 1: Nova Setup");
    
    let start = Instant::now();
    let policy = OriginPolicy::default();
    let policy_root = policy.compute_hash();
    
    let params = NovaParams::setup(policy_root)?;
    let setup_time = start.elapsed();
    
    println!("    Setup complete in {:.2}s", setup_time.as_secs_f64());
    println!();

    // ========================================
    // Step 2: Initialize
    // ========================================
    println!(" Step 2: Initialize Prover");
    
    let genesis_state = [0u8; 32];
    let mut nova_prover = NovaLineageProver::new(params);
    nova_prover.initialize(genesis_state, 0)?;
    
    // Also create witness generator
    let mut witness_gen = WitnessGenerator::new(policy.clone());
    witness_gen.reset(genesis_state);
    
    println!("    Nova prover initialized");
    println!("    Genesis commitment: {:?}", hex::encode(&nova_prover.genesis()[..8]));
    println!();

    // ========================================
    // Step 3: Prove Steps (SLOW: 100-500ms each)
    // ========================================
    println!(" Step 3: Prove Transition Steps");
    println!("    Each step takes 100-500ms...");
    println!();

    let transitions = vec![
        ("Genesis → User", OriginClass::User),
        ("User → User", OriginClass::User),
        ("User → User", OriginClass::User),
    ];

    let mut total_step_time = std::time::Duration::ZERO;

    for (i, (desc, origin)) in transitions.iter().enumerate() {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            *origin,
            (i as u64 + 1) * 1000,
        );
        
        // Generate witness
        let witness = witness_gen.generate_witness(&transition)?;
        
        // Prove step
        let start = Instant::now();
        nova_prover.prove_step(&witness)?;
        let step_time = start.elapsed();
        total_step_time += step_time;
        
        println!("    Step {}: {} ({:.2}ms)", 
            i + 1, 
            desc, 
            step_time.as_secs_f64() * 1000.0
        );
    }
    
    println!();
    println!("    Total step time: {:.2}s", total_step_time.as_secs_f64());
    println!("    Average per step: {:.2}ms", 
        (total_step_time.as_secs_f64() * 1000.0) / transitions.len() as f64
    );
    println!();

    // ========================================
    // Step 4: Verify Running SNARK
    // ========================================
    println!(" Step 4: Verify Running SNARK");
    
    let start = Instant::now();
    let valid = nova_prover.verify()?;
    let verify_time = start.elapsed();
    
    println!("    Running SNARK valid: {}", valid);
    println!("     Verification time: {:.2}ms", verify_time.as_secs_f64() * 1000.0);
    println!();

    // ========================================
    // Step 5: Compress (SLOW: 10-60 seconds)
    // ========================================
    println!("  Step 5: Compress to Final Proof");
    
    let start = Instant::now();
    let proof = nova_prover.finalize()?;
    let compression_time = start.elapsed();
    
    println!("    Compression complete in {:.2}s", compression_time.as_secs_f64());
    println!();

    // ========================================
    // Summary
    // ========================================
    println!("═══════════════════════════════════════════════════════════════");
    println!("                        SUMMARY                                 ");
    println!("═══════════════════════════════════════════════════════════════");
    println!(" Proof Details:");
    println!("   Lineage depth: {} steps", proof.num_steps);
    println!("   Proof size: {} bytes ({:.1}KB)", 
        proof.proof_size(), 
        proof.proof_size() as f64 / 1024.0
    );
    println!("    Final lineage: {}...", &proof.final_lineage.to_hex()[..16]);
    println!();
    println!("  Timing Summary:");
    println!("    Setup: {:.2}s", setup_time.as_secs_f64());
    println!("    Proving ({} steps): {:.2}s", transitions.len(), total_step_time.as_secs_f64());
    println!("    Compression: {:.2}s", compression_time.as_secs_f64());
    let total = setup_time + total_step_time + compression_time;
    println!("    Total: {:.2}s", total.as_secs_f64());
    println!();
    println!(" This proof provides REAL zero-knowledge privacy!");
    println!("   The verifier learns nothing about intermediate states.");
    println!();

    Ok(())
}