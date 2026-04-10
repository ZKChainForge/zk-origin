use zk_origin::policy::{PolicyTree, get_policy_leaves};
use zk_origin::types::OriginClass;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════╗");
    println!("║   ZK-ORIGIN Policy Verification        ║");
    println!("╚════════════════════════════════════════╝\n");

    // Build policy tree
    let transitions = get_policy_leaves();
    let tree = PolicyTree::new(transitions.clone());

    println!(" Policy Tree Status:");
    println!("   Transitions: {}", tree.leaf_count());
    println!("   Tree depth: {}", tree.get_depth());
    println!("   Root: {:?}\n", tree.root());

    loop {
        println!("Select operation:");
        println!("  1. Verify transition");
        println!("  2. List allowed transitions");
        println!("  3. Get policy root");
        println!("  4. Exit");
        println!();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => verify_transition(&tree)?,
            "2" => list_transitions(&transitions),
            "3" => show_policy_root(&tree),
            "4" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid choice\n"),
        }
    }

    Ok(())
}

fn verify_transition(
    tree: &PolicyTree,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n Verify Transition");

    println!("From class (0-6): ");
    let mut from = String::new();
    io::stdin().read_line(&mut from)?;
    let from_class = OriginClass::from_u8(from.trim().parse()?)
        .ok_or("Invalid from class")?;

    println!("To class (0-6): ");
    let mut to = String::new();
    io::stdin().read_line(&mut to)?;
    let to_class =
        OriginClass::from_u8(to.trim().parse()?).ok_or("Invalid to class")?;

    match tree.prove(from_class, to_class) {
        Some(proof) => {
            let is_valid = tree.verify(&proof);
            println!("\n Transition {} → {}", from_class, to_class);
            println!("   Proof valid: {}", is_valid);
            println!("   Path length: {}", proof.path_elements.len());
        }
        None => {
            println!("\n❌ Transition {} → {} NOT ALLOWED", from_class, to_class);
        }
    }

    println!();
    Ok(())
}

fn list_transitions(transitions: &[(OriginClass, OriginClass)]) {
    println!("\n Allowed Transitions ({} total):\n", transitions.len());

    for (from, to) in transitions {
        println!("  {} → {}", from, to);
    }

    println!();
}

fn show_policy_root(tree: &PolicyTree) {
    let root = tree.root();
    println!("   {:?}\n", root);
}