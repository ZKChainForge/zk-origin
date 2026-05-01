//! CLI for ZK-ORIGIN SDK

use clap::Parser;
use zk_origin_sdk::{ZKOrigin, Config};

#[derive(Parser)]
#[command(name = "zk-origin")]
#[command(about = "ZK-ORIGIN SDK CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Initialize configuration
    Init {
        /// Config file path
        #[arg(short, long)]
        config: Option<String>,
    },
    
    /// Generate witness
    Witness {
        /// Previous state file
        #[arg(short, long)]
        prev: String,
        
        /// New state file
        #[arg(short, long)]
        new: String,
        
        /// Output file
        #[arg(short, long)]
        output: String,
    },
    
    /// Generate proof
    Proof {
        /// Witness file
        #[arg(short, long)]
        witness: String,
        
        /// Output file
        #[arg(short, long)]
        output: String,
    },
    
    /// Submit proof
    Submit {
        /// Proof file
        #[arg(short, long)]
        proof: String,
        
        /// Public inputs file
        #[arg(short, long)]
        public: String,
    },
    
    /// Query state
    Query {
        /// State hash
        #[arg(short, long)]
        hash: String,
    },
    
    /// Get stats
    Stats,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let args = Args::parse();
    
    match args.command {
        Commands::Init { config } => {
            let cfg = Config::from_env()?;
            if let Some(path) = config {
                cfg.save(path)?;
            } else {
                cfg.save("zk-origin.json")?;
            }
            println!(" Configuration saved");
        }
        
        Commands::Witness { .. } => {
            println!(" Generating witness...");
            // TODO: Implement
        }
        
        Commands::Proof { .. } => {
            println!(" Generating proof...");
            // TODO: Implement
        }
        
        Commands::Submit { .. } => {
            println!(" Submitting proof...");
            // TODO: Implement
        }
        
        Commands::Query { hash } => {
            println!(" Querying state: {}", hash);
            // TODO: Implement
        }
        
        Commands::Stats => {
            println!(" Getting stats...");
            // TODO: Implement
        }
    }
    
    Ok(())
}