use zk_origin::policy::{PolicyTree, get_policy_leaves};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use ff_ce::PrimeField;  // ← ADD THIS LINE

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Generating Policy Merkle Tree...\n");
    
    // Get all allowed transitions
    let transitions = get_policy_leaves();
    
    println!("Allowed transitions:");
    for (from, to) in &transitions {
        println!("  {:?} → {:?}", from, to);
    }
    println!("\nTotal: {} transitions\n", transitions.len());
    
    // Build tree
    let tree = PolicyTree::new(transitions.clone());
    let root = tree.root();
    
    println!("Policy Merkle Root:");
    println!("  {:?}\n", root);
    
    // Convert root to hex for Solidity
    let root_repr = root.into_repr();
    let root_hex = format!("0x{:064x}", root_repr.0[0]);
    
    println!("Policy Merkle Root (hex for Solidity):");
    println!("  {}\n", root_hex);
    
    // Generate sample proofs
    println!("Sample Merkle Proofs:");
    for (from, to) in transitions.iter().take(3) {
        if let Some(proof) = tree.prove(*from, *to) {
            println!("\n  {:?} → {:?}:", from, to);
            println!("    Leaf: {:?}", proof.leaf);
            println!("    Path length: {}", proof.path_elements.len());
            println!("    Valid: {}", tree.verify(&proof));
        }
    }
    
    // Save to JSON for deployment script
    let transitions_array: Vec<[u8; 2]> = transitions
        .iter()
        .map(|(from, to)| [*from as u8, *to as u8])
        .collect();
    
    let output = json!({
        "root": root_hex,
        "transitions": transitions_array,
        "transition_count": transitions.len(),
        "tree_depth": 6,
    });
    
    let mut file = File::create("policy_root.json")?;
    file.write_all(serde_json::to_string_pretty(&output)?.as_bytes())?;
    
    println!("\n✅ Policy root saved to policy_root.json");
    println!("✅ Ready for contract deployment");
    
    Ok(())
}