//! Orchestrator main

use clap::Parser;
use zk_origin_orchestrator::{PipelineExecutor, Config, rpc::EthereumRPC, Error};
use log::info;

#[derive(Parser)]
#[command(name = "orchestrator")]
#[command(about = "ZK-ORIGIN Orchestrator")]
struct Args {
    /// Number of steps
    #[arg(short, long, default_value = "3")]
    steps: usize,
    
    /// Config file
    #[arg(short, long)]
    config: Option<String>,
    
    /// Verbose
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    
    let args = Args::parse();
    
    println!("\n{}", "═".repeat(60));
    println!(" ZK-ORIGIN ORCHESTRATOR");
    println!("{}\n", "═".repeat(60));
    
    // Load config
    info!("Loading configuration");
    let config = if let Some(path) = args.config {
        Config::from_file(path)?
    } else {
        Config::default()
    };
    
    // Create RPC client
    let rpc = EthereumRPC::new(
        config.rpc_endpoint.clone(),
        config.contract_address.clone(),
    );
    
    // Create executor
    let executor = PipelineExecutor::new(rpc);
    
    // Execute steps
    for step in 0..args.steps {
        println!("\n{}", "─".repeat(60));
        println!("Step {}/{}", step + 1, args.steps);
        println!("{}", "─".repeat(60));
        
        if let Err(e) = executor.execute_step(step).await {
            eprintln!(" Step failed: {}", e);
            return Err(e);
        }
    }
    
   
    
    Ok(())
}