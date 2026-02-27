//! ZK-ORIGIN Command Line Interface

use std::path::PathBuf;
use std::time::Instant;

use zk_origin::{
    LineageProver, OriginPolicy, Transition, OriginClass,
    LineageVerifier, LineageProof,
    Result, ZkOriginError,
    expected_performance,
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
        "demo-nova" => cmd_demo_nova()?,
        "prove" => cmd_prove(&args[2..])?,
        "verify" => cmd_verify(&args[2..])?,
        "benchmark" => cmd_benchmark()?,
        "benchmark-nova" => cmd_benchmark_nova()?,
        "info" => cmd_info(),
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
╔═══════════════════════════════════════════════════════════════╗
║                       ZK-ORIGIN CLI                           ║
║          Zero-Knowledge State Lineage Verification            ║
╚═══════════════════════════════════════════════════════════════╝

USAGE:
    zk-origin-cli <COMMAND> [OPTIONS]

COMMANDS:
    demo           Run commitment mode demonstration (fast)
    demo-nova      Run Nova ZK mode demonstration (slow, real proofs)
    prove          Generate a lineage proof
    verify         Verify a lineage proof
    benchmark      Run commitment mode benchmarks
    benchmark-nova Run Nova mode benchmarks (very slow)
    info           Show library information and performance estimates
    help           Show this help message
    version        Show version information

EXAMPLES:
    zk-origin-cli demo
    zk-origin-cli demo-nova
    zk-origin-cli prove --output proof.json
    zk-origin-cli verify --proof proof.json
    zk-origin-cli benchmark
    zk-origin-cli info

OPTIONS:
    -h, --help      Show help information
    -v, --version   Show version information
"#);
}

fn print_version() {
    println!("zk-origin-cli v{}", zk_origin::VERSION);
    println!("Zero-Knowledge State Lineage Verification");
}

fn cmd_info() {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    ZK-ORIGIN Information                      ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    let perf = expected_performance();
    
    println!(" Commitment Mode (hash-based, no ZK privacy):");
    println!("    Add transition:   ~{}µs", perf.commitment_mode.add_transition_us);
    println!("    Finalize:         ~{}µs", perf.commitment_mode.finalize_us);
    println!("    Verify:           ~{}µs", perf.commitment_mode.verify_us);
    println!("    Proof size:       {} bytes", perf.commitment_mode.proof_size_bytes);
    println!();
    
    println!(" Nova Mode (real ZK proofs):");
    println!("    Setup:            ~{} seconds (one-time)", perf.nova_mode.setup_seconds);
    println!("    Per step:         ~{}ms", perf.nova_mode.step_ms);
    println!("    Compression:      ~{} seconds", perf.nova_mode.compression_seconds);
    println!("    Verify:           ~{}ms", perf.nova_mode.verify_ms);
    println!("    Proof size:       ~{}KB", perf.nova_mode.proof_size_bytes / 1000);
    println!();
    
    println!(" Origin Classes:");
    for class in OriginClass::all() {
        println!("   {} = {} (rate limit: {})", 
            *class as u8,
            class,
            class.default_rate_limit()
        );
    }
    println!();
    
    println!("  Configuration:");
    println!("    Origin classes:   {}", zk_origin::NUM_ORIGIN_CLASSES);
    println!("    Policy tree depth: {}", zk_origin::POLICY_TREE_DEPTH);
    println!("    Max lineage depth: {}", zk_origin::MAX_LINEAGE_DEPTH);
}

fn cmd_demo() -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║              ZK-ORIGIN DEMO (Commitment Mode)                 ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    // Step 1: Create policy
    println!(" Step 1: Creating Origin Policy");
    let policy = OriginPolicy::default();
    println!("    Policy created with {} allowed transitions", policy.num_allowed());
    println!("    Epoch duration: {} seconds (24 hours)", policy.epoch_duration);
   

    // Step 2: Create prover
    println!(" Step 2: Initializing Lineage Prover");
    let mut prover = LineageProver::new(policy.clone())?;
    let genesis_hash = [0u8; 32];
    // ... continuing cmd_demo() function

    prover.initialize(genesis_hash)?;
    println!("    Prover created successfully");
    println!("    Genesis state initialized");
    println!("    Genesis commitment: {}...", 
        hex::encode(&prover.current_lineage().unwrap().value[..8]));
    println!();

    // Step 3: Add transitions
    println!(" Step 3: Adding State Transitions");
    
    let transitions = vec![
        ("Genesis → User", OriginClass::User, "User initiated first action"),
        ("User → User", OriginClass::User, "User continued operations"),
        ("User → User", OriginClass::User, "User completed workflow"),
    ];

    let start = Instant::now();
    for (i, (desc, origin, _note)) in transitions.iter().enumerate() {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            *origin,
            (i as u64 + 1) * 1000,
        );
        
        match prover.add_transition(t) {
            Ok(_) => println!("    Transition {}: {}", i + 1, desc),
            Err(e) => println!("    Transition {}: {} - Error: {}", i + 1, desc, e),
        }
    }
    let transition_time = start.elapsed();
    
    println!("     Transitions added in {:?}", transition_time);
    println!("    Current lineage depth: {}", prover.current_depth());

    // Step 4: Generate proof
    println!(" Step 4: Generating Lineage Proof");
    let start = Instant::now();
    let proof = prover.finalize()?;
    let proof_time = start.elapsed();
    
    println!("    Proof generated successfully!");
    println!("    Proof Details:");
    println!("       Lineage depth: {} transitions", proof.num_steps);
    println!("       Proof size: {} bytes", proof.proof_size());
    println!("       Generation time: {:?}", proof_time);
    println!("       Final lineage: {}...", &proof.final_lineage.to_hex()[..16]);
    println!("       Genesis: {}...", &proof.genesis_commitment.to_hex()[..16]);
    println!();

    // Step 5: Verify proof
    println!(" Step 5: Verifying Lineage Proof");
    let verifier = LineageVerifier::new(genesis_hash, &policy);
    
    let start = Instant::now();
    let verification_result = verifier.verify(&proof);
    let verify_time = start.elapsed();
    
    match verification_result {
        Ok(true) => {
            println!("    PROOF IS VALID!");
            println!("    Verification Details:");
            let detailed = verifier.verify_detailed(&proof);
            println!("       Genesis check:  {}", if detailed.genesis_valid { " PASSED" } else { " FAILED" });
            println!("       Policy check:   {}", if detailed.policy_valid { " PASSED" } else { "FAILED" });
            println!("       Depth check:    {}", if detailed.depth_valid { " PASSED" } else { " FAILED" });
            println!("       Proof check:    {}", if detailed.proof_valid { " PASSED" } else { " FAILED" });
            println!("       Verification time: {:?}", verify_time);
        }
        Ok(false) => {
            println!("    PROOF IS INVALID!");
        }
        Err(e) => {
            println!("    Verification error: {}", e);
        }
    }
    println!();

    // Step 6: Test policy violation
    println!("  Step 6: Testing Policy Enforcement");
    let mut test_prover = LineageProver::new(policy.clone())?;
    test_prover.initialize([0u8; 32])?;
    
    // Valid transition
    let valid_t = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
    test_prover.add_transition(valid_t)?;
    println!("    Valid: Genesis → User (allowed)");
    
    // Invalid transition (User → Admin not allowed in default policy)
    let invalid_t = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
    match test_prover.add_transition(invalid_t) {
        Ok(_) => println!("   ? User → Admin succeeded (unexpected)"),
        Err(ZkOriginError::PolicyViolation { from, to }) => {
            println!("    Invalid: {} → {} (correctly rejected)", from, to);
        }
        Err(e) => println!("    Error: {}", e),
    }
    Ok(())
}

fn cmd_demo_nova() -> Result<()> {
    println!("Press Enter to continue or Ctrl+C to cancel...");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    
    use zk_origin::prover::nova_prover::{NovaParams, NovaLineageProver};
    
    // Step 1: Setup
    println!(" Step 1: Setting up Nova public parameters...");
    
    let start = Instant::now();
    let policy_root = [0u8; 32]; // Simplified for demo
    let params = NovaParams::setup(policy_root)?;
    let setup_time = start.elapsed();
    
    println!("    Setup complete in {:.2}s", setup_time.as_secs_f64());
    println!();
    
    // Step 2: Initialize prover
    println!(" Step 2: Initializing Nova prover...");
    let mut prover = NovaLineageProver::new(params);
    prover.initialize([0u8; 32], 0)?;
    println!("    Prover initialized with genesis state");
    
    // Step 3: Add steps
    println!(" Step 3: Adding transition steps...");
    
    // We need to create witnesses for Nova
    // For a real demo, you'd use the WitnessGenerator
    println!("   • Nova setup is expensive (~{}s) but done once", setup_time.as_secs());
    
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

    let use_nova = args.iter().any(|a| a == "--nova");


    println!(" Generating Lineage Proof");
    println!("    Output: {}", output_path.display());
    println!("    Transitions: {}", num_transitions);
    println!("    Mode: {}", if use_nova { "Nova (ZK)" } else { "Commitment" });

    if use_nova {
        println!("  Nova mode requested. This will be slow...");
        // Nova implementation would go here
        println!("   Nova proving not yet implemented in CLI.");
        println!("   Use the library directly for Nova proofs.");
        return Ok(());
    }

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
        print!("\r    Adding transitions... {}/{}", i + 1, num_transitions);
    }
    println!();

    let proof = prover.finalize()?;
    let duration = start.elapsed();

    // Save proof
    let json = proof.to_json()?;
    std::fs::write(&output_path, &json)?;

    println!();
    println!(" Proof Generated");
    println!("    Saved to: {}", output_path.display());
    println!("    Depth: {} transitions", proof.num_steps);
    println!("    Size: {} bytes", proof.proof_size());
    println!("    Time: {:?}", duration);
    println!();

    Ok(())
}

fn cmd_verify(args: &[String]) -> Result<()> {
    let proof_path = args.iter()
        .position(|a| a == "--proof" || a == "-p")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("proof.json"));

    println!();
    println!(" Verifying Lineage Proof");
    println!("    Input: {}", proof_path.display());
    println!();

    // Load proof
    let json = std::fs::read_to_string(&proof_path)?;
    let proof = LineageProof::from_json(&json)?;

    println!("Proof loaded:");
    println!("    Depth: {} transitions", proof.num_steps);
    println!("    Size: {} bytes", proof.proof_size());
    println!("    Lineage: {}...", &proof.final_lineage.to_hex()[..16]);
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
            println!("    Status: VALID");
            println!("    Time: {:?}", duration);
        }
        Ok(false) => {
            println!(" Verification Result");
            println!("    Status: INVALID");
        }
        Err(e) => {
            println!(" Verification Result");
            println!("    Error: {}", e);
        }
    }
    println!();

    Ok(())
}

fn cmd_benchmark() -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║              ZK-ORIGIN BENCHMARKS (Commitment Mode)           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
   

    let policy = OriginPolicy::default();

    // Benchmark: Prover initialization
    println!(" Benchmark: Prover Initialization");
    let start = Instant::now();
    for _ in 0..100 {
        let mut prover = LineageProver::new(policy.clone())?;
        prover.initialize([0u8; 32])?;
    }
    let duration = start.elapsed();
    println!("    100 initializations: {:?}", duration);
    println!("    Average: {:?}", duration / 100);
    println!();

    // Benchmark: Add transitions
    println!(" Benchmark: Add Transitions");
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
    println!("    1000 transitions: {:?}", duration);
    println!("    Average per transition: {:?}", duration / 1000);
    println!("   Throughput: {:.0} transitions/sec", 1000.0 / duration.as_secs_f64());
    println!();

    // Benchmark: Proof generation at different depths
    println!(" Benchmark: Proof Generation");
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
        
        println!("    Depth {:>4}: {:>10?}  (proof size: {} bytes)", 
                 depth, duration, proof.proof_size());
    }
    println!();

    // Benchmark: Verification
    println!(" Benchmark: Proof Verification");
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
        
        println!("    Depth {:>4}: {:>10?} (1000 verifications)", depth, duration);
    }

    Ok(())
}

fn cmd_benchmark_nova() -> Result<()> {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║               ZK-ORIGIN BENCHMARKS (Nova Mode)                ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!("Press Enter to continue or Ctrl+C to cancel...");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    use zk_origin::prover::nova_prover::NovaParams;
    
    // Benchmark: Setup
    println!(" Benchmark: Nova Setup");
    let start = Instant::now();
    let policy_root = [0u8; 32];
    let _params = NovaParams::setup(policy_root)?;
    let setup_time = start.elapsed();
    
    println!("    Setup time: {:.2}s", setup_time.as_secs_f64());
    println!();

    println!(" Benchmark: Nova Step Proving");
    println!("     Full step proving requires witness generation.");
    println!("    Setup overhead: {:.2}s", setup_time.as_secs_f64());
    println!("    Expected per-step time: 100-500ms");
    println!();

    
    

    Ok(())
}
    