//! Production CLI for state operations

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use zk_origin_core::{OriginClass, OriginPolicy, State, StateData, Transition};

#[derive(Parser)]
#[command(name = "zk-origin-core")]
#[command(about = "ZK-ORIGIN State Machine CLI", version = env!("CARGO_PKG_VERSION"))]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(global = true, long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Create genesis state
    Genesis {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Create new state
    NewState {
        #[arg(short, long)]
        nonce: u64,

        #[arg(short, long)]
        timestamp: u64,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate state
    Validate {
        #[arg(short, long)]
        state: PathBuf,
    },

    /// Create transition
    Transition {
        #[arg(short, long)]
        prev_state: PathBuf,

        #[arg(short, long)]
        new_state: PathBuf,

        #[arg(short, long)]
        nonce: u64,

        #[arg(short, long)]
        initiator: String,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Get state info
    Info {
        #[arg(short, long)]
        state: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    env_logger::Builder::from_default_env()
        .filter_level(args.log_level.parse()?)
        .init();

    match args.command {
        Commands::Genesis { output } => {
            let genesis = State::genesis(StateData::default())?;
            let json = serde_json::to_string_pretty(&genesis)?;

            if let Some(path) = output {
                std::fs::write(&path, json)?;
                println!("Genesis state saved to {}", path.display());
            } else {
                println!("{}", json);
            }
        }

        Commands::NewState {
            nonce,
            timestamp,
            output,
        } => {
            let state = State::new(StateData::default(), timestamp, nonce)?;
            let json = serde_json::to_string_pretty(&state)?;

            if let Some(path) = output {
                std::fs::write(&path, json)?;
                println!("State saved to {}", path.display());
            } else {
                println!("{}", json);
            }
        }

        Commands::Validate { state } => {
            let json = std::fs::read_to_string(&state)?;
            let state: State = serde_json::from_str(&json)?;

            match state.validate() {
                Ok(_) => println!("State is valid"),
                Err(e) => println!("State is invalid: {}", e),
            }
        }

        Commands::Info { state } => {
            let json = std::fs::read_to_string(&state)?;
            let state: State = serde_json::from_str(&json)?;

            println!("State Info:");
            println!("  Hash: {}", state.hash);
            println!("  Nonce: {}", state.nonce);
            println!("  Timestamp: {}", state.timestamp);
        }

        Commands::Transition {
            prev_state,
            new_state,
            nonce,
            initiator,
            output,
        } => {
            let prev_json = std::fs::read_to_string(&prev_state)?;
            let new_json = std::fs::read_to_string(&new_state)?;

            let prev_state: State = serde_json::from_str(&prev_json)?;
            let new_state: State = serde_json::from_str(&new_json)?;

            let transition = Transition::new(
                prev_state,
                new_state,
                OriginClass::User,
                OriginClass::User,
                initiator,
                nonce,
            )?;

            let json = serde_json::to_string_pretty(&transition)?;

            if let Some(path) = output {
                std::fs::write(&path, json)?;
                println!("Transition saved to {}", path.display());
            } else {
                println!("{}", json);
            }
        }
    }

    Ok(())
}
