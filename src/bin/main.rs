use clap::{Parser, Subcommand};
use zk_origin::policy::{PolicyTree, get_policy_leaves};

#[derive(Parser)]
#[command(name = "zk-origin")]
#[command(about = "ZK-ORIGIN CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate policy Merkle tree
    GeneratePolicy,
    
    /// Verify a policy proof
    VerifyProof {
        /// From origin class (0-6)
        #[arg(short, long)]
        from: u8,
        
        /// To origin class (0-6)
        #[arg(short, long)]
        to: u8,
    },
    
    /// Show policy transitions
    ShowPolicy,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::GeneratePolicy => {
            generate_policy()?;
        }
        Commands::VerifyProof { from, to } => {
            verify_proof(from, to)?;
        }
        Commands::ShowPolicy => {
            show_policy();
        }
    }
    
    Ok(())
}

fn generate_policy() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Generating Policy Merkle Tree...\n");
    
    let transitions = get_policy_leaves();
    let tree = PolicyTree::new(transitions.clone());
    let root = tree.root();
    
    println!("Policy root: {}", root);
    println!("Transitions: {}", transitions.len());
    
    Ok(())
}

fn verify_proof(from: u8, to: u8) -> Result<(), Box<dyn std::error::Error>> {
    use zk_origin::types::OriginClass;
    
    let from_class = OriginClass::from_u8(from).ok_or("Invalid from class")?;
    let to_class = OriginClass::from_u8(to).ok_or("Invalid to class")?;
    
    let transitions = get_policy_leaves();
    let tree = PolicyTree::new(transitions);
    
    if let Some(proof) = tree.prove(from_class, to_class) {
        println!("✅ Proof found for {} → {}", from_class, to_class);
        println!("Valid: {}", tree.verify(&proof));
    } else {
        println!("❌ No proof found (transition not allowed)");
    }
    
    Ok(())
}

fn show_policy() {
    use zk_origin::policy::default_policy_matrix;
    use zk_origin::types::OriginClass;
    
    println!("📋 ZK-ORIGIN Policy Matrix\n");
    
    let policy = default_policy_matrix();
    
    for from in OriginClass::all() {
        println!("{:?}:", from);
        for to in OriginClass::all() {
            if policy.get(&(from, to)).copied().unwrap_or(false) {
                println!("  → {:?} ✓", to);
            }
        }
        println!();
    }
}