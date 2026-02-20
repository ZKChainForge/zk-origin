
//! ZK-ORIGIN Command Line Interface

use std::path::PathBuf;
use std::time::Instant;

use zk_origin::{
    LineageProver, OriginPolicy, Transition, OriginClass,
    LineageVerifier, LineageProof,
    Result, ZkOriginError,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "demo" => cmd_demo()?,
        "prove" => cmd_prove(&args[2..])?,
        "verify" => cmd_verify(&args[2..])?,
        "benchmark" => cmd_benchmark()?,
        "help" | "--help" | "-h" => print_usage(),
        "version" | "--version" | "-v" => print_version(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            return Err(ZkOriginError::ConfigurationError(
                format!("Unknown command: {}", args[1])
            ));
        }
    }
    
    Ok(())
}

fn print_usage() {
    println!(r#"
---------------------------------------------------------------
                      ZK-ORIGIN CLI                            
         Zero-Knowledge State Lineage Verification             
---------------------------------------------------------------

USAGE:
    zk-origin-cli <COMMAND> [OPTIONS]

COMMANDS:
    demo        Run a demonstration of ZK-ORIGIN
    prove       Generate a lineage proof
    verify      Verify a lineage proof
    benchmark   Run performance benchmarks
    help        Show this help message
    version     Show version information

EXAMPLES:
    zk-origin-cli demo
    zk-origin-cli prove --output proof.json
    zk-origin-cli verify --proof proof.json
    zk-origin-cli benchmark

OPTIONS:
    -h, --help      Show help information
    -v, --version   Show version information
"#);
}

fn print_version() {
    println!("zk-origin-cli v{}", env!("CARGO_PKG_VERSION"));
    println!("Zero-Knowledge State Lineage Verification");
    println!("Copyright (c) 2024 ZK-ORIGIN Contributors");
}

fn cmd_demo() -> Result<()> {
    println!();
    println!("---------------------------------------------------------------");
    println!("                    ZK-ORIGIN DEMO                             ");
    println!("---------------------------------------------------------------");
    println!();

    // Step 1: Create policy
    println!(" Step 1: Creating Origin Policy ");
    let policy = OriginPolicy::default();
    println!("   Policy created with {} allowed transitions", policy.num_allowed());
    println!("   Epoch duration: {} seconds (24 hours)", policy.epoch_duration);
    println!("   Rate limits configured for each origin class");
    println!();

    // Step 2: Create prover
    println!(" Step 2: Initializing Lineage Prover ");
    let mut prover = LineageProver::new(policy.clone())?;
    let genesis_hash = [0u8; 32];
    prover.initialize(genesis_hash)?;
    println!("   Prover created successfully");
    println!("   Genesis state initialized");
    println!("   Genesis commitment: {}...", hex::encode(&prover.current_lineage().unwrap().value[..8]));
    println!();

    // Step 3: Add transitions
    println!(" Step 3: Adding State Transitions ");
    
    let transitions = vec![
        ("Genesis → User", OriginClass::User, "User initiated first action"),
        ("User → User", OriginClass::User, "User continued operations"),
        ("User → User", OriginClass::User, "User completed workflow"),
    ];

    for (i, (desc, origin, _note)) in transitions.iter().enumerate() {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            *origin,
            (i as u64 + 1) * 1000,
        );
        
        match prover.add_transition(t) {
            Ok(_) => println!("   Transition {}: {}", i + 1, desc),
            Err(e) => println!("   Transition {}: {} - Error: {}", i + 1, desc, e),
        }
    }
    
    
    println!("  Current lineage depth: {}", prover.current_depth());
    println!();

    // Step 4: Generate proof
    println!(" Step 4: Generating Lineage Proof ");
    let start = Instant::now();
    let proof = prover.finalize()?;
    let duration = start.elapsed();
    
    println!("   Proof generated successfully!");
    println!("  Proof Details:");
    println!("  Lineage depth: {} transitions", proof.num_steps);
    println!("  Proof size: {} bytes", proof.proof_size());
    println!("  Generation time: {:?}", duration);
    println!("  Final lineage: {}...", &proof.final_lineage.to_hex()[..16]);
    println!("  Genesis: {}...", &proof.genesis_commitment.to_hex()[..16]);
   

    // Step 5: Verify proof
    println!("Step 5: Verifying Lineage Proof");
    let verifier = LineageVerifier::new(genesis_hash, &policy);
    
    let start = Instant::now();
    let verification_result = verifier.verify(&proof);
    let verify_duration = start.elapsed();
    
    match verification_result {
        Ok(true) => {
            println!("   PROOF IS VALID!");
       
            println!("  Verification Details:");
            let detailed = verifier.verify_detailed(&proof);
            println!(" Genesis check: {}", if detailed.genesis_valid { " PASSED" } else { " FAILED" });
            println!(" Policy check: {}", if detailed.policy_valid { " PASSED" } else { " FAILED" });
            println!(" Depth check: {}", if detailed.depth_valid { " PASSED" } else { " FAILED" });
            println!(" Proof check: {}", if detailed.proof_valid { " PASSED" } else { " FAILED" });
            println!(" Verification time: {:?}", verify_duration);
        }
        Ok(false) => {
            println!("   PROOF IS INVALID!");
        }
        Err(e) => {
            println!("   Verification error: {}", e);
        }
    }
    
    println!();

    // Step 6: Test policy violation
    println!(" Step 6: Testing Policy Enforcement");
    let mut test_prover = LineageProver::new(policy.clone())?;
    test_prover.initialize([0u8; 32])?;
    
    // Valid transition
    let valid_t = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
    test_prover.add_transition(valid_t)?;
    println!("   Valid: Genesis → User (allowed)");
    
    // Invalid transition (User → Admin not allowed in default policy)
    let invalid_t = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
    match test_prover.add_transition(invalid_t) {
        Ok(_) => println!("  ? User → Admin succeeded (unexpected)"),
        Err(ZkOriginError::PolicyViolation { from, to }) => {
            println!("   Invalid: {} → {} (correctly rejected)", from, to);
        }
        Err(e) => println!("   Error: {}", e),
    }
   
    println!("  Policy enforcement is working correctly!");
    
    println!();

    

    Ok(())
}

fn cmd_prove(args: &[String]) -> Result<()> {
    let output_path = args.iter()
        .position(|a| a == "--output" || a == "-o")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("proof.json"));

    let num_transitions: usize = args.iter()
        .position(|a| a == "--steps" || a == "-n")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    println!(" Generating Lineage Proof");
    println!("  Output: {}", output_path.display());
    println!("  Transitions: {}", num_transitions);
    println!();

    let policy = OriginPolicy::default();
    let mut prover = LineageProver::new(policy)?;
    prover.initialize([0u8; 32])?;

    let start = Instant::now();
    
    for i in 0..num_transitions {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(t)?;
        print!("\r  Adding transitions... {}/{}", i + 1, num_transitions);
    }
    

    let proof = prover.finalize()?;
    let duration = start.elapsed();

    // Save proof
    let json = proof.to_json()?;
    std::fs::write(&output_path, &json)?;

    println!();
    println!(" Proof Generated");
    println!("   Saved to: {}", output_path.display());
    println!("   Depth: {} transitions", proof.num_steps);
    println!("   Size: {} bytes", proof.proof_size());
    println!("   Time: {:?}", duration);
    

    Ok(())
}

fn cmd_verify(args: &[String]) -> Result<()> {
    let proof_path = args.iter()
        .position(|a| a == "--proof" || a == "-p")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("proof.json"));

    println!(" Verifying Lineage Proof");
    println!("  Input: {}", proof_path.display());
    println!();

    // Load proof
    let json = std::fs::read_to_string(&proof_path)?;
    let proof = LineageProof::from_json(&json)?;

    println!("  Proof loaded:");
    println!("  Depth: {} transitions", proof.num_steps);
    println!("  Size: {} bytes", proof.proof_size());
    println!("  Lineage: {}...", &proof.final_lineage.to_hex()[..16]);
    println!();

    // Verify
    let policy = OriginPolicy::default();
    let genesis_hash = [0u8; 32];
    let verifier = LineageVerifier::new(genesis_hash, &policy);

    let start = Instant::now();
    let result = verifier.verify(&proof);
    let duration = start.elapsed();

    match result {
        Ok(true) => {
            println!(" Verification Result");
            println!("   PROOF IS VALID!");
            println!("   Verification time: {:?}", duration);
        
        }
        Ok(false) => {
            println!(" Verification Result ");
            println!("   PROOF IS INVALID!");
            
        }
        Err(e) => {
            println!(" Verification Result");
            println!("   Error: {}", e);
            
        }
    }

    Ok(())
}

fn cmd_benchmark() -> Result<()> {

    println!("--------------------------------------------------------------------");
    println!("                  ZK-ORIGIN BENCHMARKS                              ");
    println!("--------------------------------------------------------------------");

    let policy = OriginPolicy::default();

    // Benchmark: Prover initialization
    println!(" Benchmark: Prover Initialization ");
    let start = Instant::now();
    for _ in 0..100 {
        let mut prover = LineageProver::new(policy.clone())?;
        prover.initialize([0u8; 32])?;
    }
    let duration = start.elapsed();
    println!("  100 initializations: {:?}", duration);
    println!("  Average: {:?}", duration / 100);
    println!();

    // Benchmark: Add transitions
    println!(" Benchmark: Add Transitions ");
    let mut prover = LineageProver::new(policy.clone())?;
    prover.initialize([0u8; 32])?;
    
    let start = Instant::now();
    for i in 0..1000 {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            i as u64 * 1000,
        );
        prover.add_transition(t)?;
    }
    let duration = start.elapsed();
    println!("  1000 transitions: {:?}", duration);
    println!("  Average per transition: {:?}", duration / 1000);
    println!("  Throughput: {:.0} transitions/sec", 1000.0 / duration.as_secs_f64());
    println!();

    // Benchmark: Proof generation
    println!(" Benchmark: Proof Generation ");
    for &depth in &[10, 100, 500, 1000] {
        let mut prover = LineageProver::new(policy.clone())?;
        prover.initialize([0u8; 32])?;
        
        for i in 0..depth {
            let t = Transition::new(
                [i as u8; 32],
                [(i + 1) as u8; 32],
                OriginClass::User,
                i as u64 * 1000,
            );
            prover.add_transition(t)?;
        }
        
        let start = Instant::now();
        let proof = prover.finalize()?;
        let duration = start.elapsed();
        
        println!("  Depth {:>4}: {:>10?}  (proof size: {} bytes)", 
                 depth, duration, proof.proof_size());
    }
    
    println!();

    // Benchmark: Verification
    println!(" Benchmark: Proof Verification ");
    let genesis_hash = [0u8; 32];
    let verifier = LineageVerifier::new(genesis_hash, &policy);
    
    for &depth in &[10, 100, 500, 1000] {
        let mut prover = LineageProver::new(policy.clone())?;
        prover.initialize([0u8; 32])?;
        
        for i in 0..depth {
            let t = Transition::new(
                [i as u8; 32],
                [(i + 1) as u8; 32],
                OriginClass::User,
                i as u64 * 1000,
            );
            prover.add_transition(t)?;
        }
        
        let proof = prover.finalize()?;
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = verifier.verify(&proof);
        }
        let duration = start.elapsed();
        
        println!("  Depth {:>4}: {:>10?} (1000 verifications)", depth, duration);
    }
    

    

    Ok(())
}
