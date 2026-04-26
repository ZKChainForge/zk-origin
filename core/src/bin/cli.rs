//! Core CLI for state machine operations

use clap::{Parser, Subcommand};
use zk_origin_core::{State, StateData, Transition, StateMachine, OriginPolicy};

#[derive(Parser)]
#[command(name = "zk-origin-core")]
#[command(about = "ZK-ORIGIN Core State Machine CLI")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create genesis state
    Genesis {
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Create new state
    NewState {
        /// Nonce
        #[arg(short, long)]
        nonce: u64,
        
        /// Timestamp
        #[arg(short, long)]
        timestamp: u64,
        
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Validate transition
    Validate {
        /// Previous state file
        #[arg(short, long)]
        prev: String,
        
        /// New state file
        #[arg(short, long)]
        new: String,
    },
    
    /// Get state info
    Info {
        /// State file
        #[arg(short, long)]
        state: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let args = Args::parse();
    
    match args.command {
        Commands::Genesis { output } => {
            let genesis = State::genesis(StateData::default());
            
            let json = serde_json::to_string_pretty(&genesis)?;
            
            if let Some(path) = output {
                std::fs::write(&path, json)?;
                println!("✅ Genesis state saved to {}", path);
            } else {
                println!("{}", json);
            }
        }
        
        Commands::NewState { nonce, timestamp, output } => {
            let state = State::new(StateData::default(), timestamp, nonce);
            
            let json = serde_json::to_string_pretty(&state)?;
            
            if let Some(path) = output {
                std::fs::write(&path, json)?;
                println!("✅ State saved to {}", path);
            } else {
                println!("{}", json);
            }
        }
        
        Commands::Validate { prev, new } => {
            let prev_json = std::fs::read_to_string(&prev)?;
            let new_json = std::fs::read_to_string(&new)?;
            
            let prev_state: State = serde_json::from_str(&prev_json)?;
            let new_state: State = serde_json::from_str(&new_json)?;
            
            let transition = Transition::new(prev_state, new_state, "cli".to_string(), 1)?;
            let policy = OriginPolicy::default();
            
            if transition.is_valid(&policy) {
                println!("✅ Transition is valid");
            } else {
                println!("❌ Transition is invalid");
            }
        }
        
        Commands::Info { state } => {
            let json = std::fs::read_to_string(&state)?;
            let state: State = serde_json::from_str(&json)?;
            
            println!("State Info:");
            println!("  Hash: 0x{}", hex::encode(&state.hash[..8]));
            println!("  Nonce: {}", state.nonce);
            println!("  Timestamp: {}", state.timestamp);
        }
    }
    
    Ok(())
}