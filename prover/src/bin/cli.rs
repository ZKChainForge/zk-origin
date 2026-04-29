//! Prover CLI

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "prover")]
#[command(about = "ZK-ORIGIN Prover CLI")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate witness
    Witness {
        /// Previous state
        #[arg(short, long)]
        prev: String,
        
        /// New state
        #[arg(short, long)]
        new: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Generate proof
    Proof {
        /// Witness file
        #[arg(short, long)]
        witness: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Verify proof
    Verify {
        /// Proof file
        #[arg(short, long)]
        proof: String,
        
        /// Public signals file
        #[arg(short, long)]
        public: String,
        
        /// Verification key file
        #[arg(short, long)]
        vk: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let args = Args::parse();
    
    match args.command {
        Commands::Witness { prev, new, output } => {
            println!(" Generating witness from {} and {}", prev, new);
            if let Some(out) = output {
                println!("   Output: {}", out);
            }
        }
        
        Commands::Proof { witness, output } => {
            println!(" Generating proof from {}", witness);
            if let Some(out) = output {
                println!("   Output: {}", out);
            }
        }
        
        Commands::Verify { proof, public: _, vk: _ } => {
            println!("  Verifying proof: {}", proof);
        }
    }
    
    Ok(())
}