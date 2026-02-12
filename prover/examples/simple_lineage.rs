//! Simple lineage example

use zk_origin_prover::{LineageProver, Origin, Transition};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ZK-ORIGIN Simple Lineage Example\n");
    
    // Create genesis state
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state)?;
    
    println!("Created prover with genesis state");
    println!("Initial lineage: {}\n", prover.current_lineage());
    
    // Add some transitions
    let transitions = vec![
        ("Deploy", Origin::Admin),
        ("User Deposit", Origin::User),
        ("User Swap", Origin::User),
        ("User Withdraw", Origin::User),
    ];
    
    let mut prev_state = genesis_state;
    
    for (i, (name, origin)) in transitions.iter().enumerate() {
        let new_state = [(i + 1) as u8; 32];
        
        println!("Transition {}: {} ({} -> {})", 
                 i + 1, 
                 name, 
                 if i == 0 { "Genesis" } else { &transitions[i-1].1.name() },
                 origin.name());
        
        prover.add_transition(Transition::new(
            prev_state,
            new_state,
            *origin,
            (i + 1) as u64 * 1000,
        ))?;
        
        println!("  Lineage: {}", prover.current_lineage());
        
        prev_state = new_state;
    }
    
    println!("\n All transitions successful!");
    println!("Final depth: {}", prover.current_lineage().depth);
    
    // Generate proof
    println!("\nGenerating proof...");
    let proof = prover.finalize()?;
    println!("Proof generated: {} bytes", proof.proof_data.len());
    println!("Proof valid: {}", proof.verify());
    
    Ok(())
}