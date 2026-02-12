//! Full demo with attack simulation

use zk_origin_prover::{LineageProver, Origin, Transition};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" ZK-ORIGIN Full Demo\n");
    println!("=".repeat(50));
    
    // Part 1: Valid flow
    println!("\n Part 1: Valid Protocol Lifecycle\n");
    
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state)?;
    
    let valid_transitions = vec![
        (Origin::Admin, "Protocol initialization"),
        (Origin::User, "First user deposit"),
        (Origin::User, "User swap"),
        (Origin::User, "Another deposit"),
    ];
    
    let mut prev_state = genesis_state;
    let mut prev_origin = Origin::Genesis;
    
    for (i, (origin, description)) in valid_transitions.iter().enumerate() {
        let new_state = [(i + 1) as u8; 32];
        
        println!("  Step {}: {} -> {}", i + 1, prev_origin, origin);
        println!("          {}", description);
        
        prover.add_transition(Transition::new(
            prev_state,
            new_state,
            *origin,
            (i + 1) as u64 * 1000,
        ))?;
        
        prev_state = new_state;
        prev_origin = *origin;
    }
    
    println!("\n   All valid transitions completed!");
    println!("  Final lineage: {}", prover.current_lineage());
    
    // Part 2: Attack simulation
    println!("\n" + &"=".repeat(50));
    println!("\n Part 2: Attack Simulation\n");
    
    println!("  Attempting privilege escalation (User -> Admin)...");
    
    let attack_result = prover.add_transition(Transition::new(
        prev_state,
        [99u8; 32],
        Origin::Admin, // ATTACK!
        9999,
    ));
    
    match attack_result {
        Ok(_) => println!("   Attack succeeded (this shouldn't happen!)"),
        Err(e) => println!("   Attack blocked: {}", e),
    }
    
    // Part 3: Valid admin flow
    println!("\n" + &"=".repeat(50));
    println!("\n Part 3: Proper Admin Flow\n");
    
    // Start fresh for admin demo
    let mut admin_prover = LineageProver::new([100u8; 32])?;
    
    let admin_flow = vec![
        (Origin::Admin, "Initial admin setup"),
        (Origin::Admin, "Configure parameters"),
        (Origin::Bridge, "Enable bridge"),
        (Origin::User, "Bridge deposit arrives"),
    ];
    
    let mut prev_state = [100u8; 32];
    let mut prev_origin = Origin::Genesis;
    
    for (i, (origin, description)) in admin_flow.iter().enumerate() {
        let new_state = [(101 + i) as u8; 32];
        
        println!("  Step {}: {} -> {}", i + 1, prev_origin, origin);
        println!("          {}", description);
        
        admin_prover.add_transition(Transition::new(
            prev_state,
            new_state,
            *origin,
            (i + 1) as u64 * 1000,
        ))?;
        
        prev_state = new_state;
        prev_origin = *origin;
    }
    
    println!("\n   Admin flow completed!");
    println!("  Final lineage: {}", admin_prover.current_lineage());
    
    // Generate final proof
    println!("\n" + &"=".repeat(50));
    println!("\nGenerating Final Proofs\n");
    
    let proof1 = prover.finalize()?;
    let proof2 = admin_prover.finalize()?;
    
    println!("  User flow proof: {} bytes, valid: {}", 
             proof1.proof_data.len(), 
             proof1.verify());
    println!("  Admin flow proof: {} bytes, valid: {}", 
             proof2.proof_data.len(), 
             proof2.verify());
    
    println!("\n Demo complete!");
    
    Ok(())
}