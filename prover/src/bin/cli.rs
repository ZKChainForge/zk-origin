//! ZK-ORIGIN CLI

use std::time::Instant;
use zk_origin::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "demo" => run_demo(),
        "benchmark" => run_benchmark(),
        "help" | "-h" | "--help" => print_help(),
        "version" | "-v" | "--version" => print_version(),
        "mode" => print_mode(),
        _ => {
            println!("Unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════╗
║                    ZK-ORIGIN CLI                              ║
║       Zero-Knowledge State Lineage Verification               ║
╚═══════════════════════════════════════════════════════════════╝

USAGE:
    zk-origin-cli <COMMAND>

COMMANDS:
    demo        Run a demonstration
    benchmark   Run performance benchmarks
    mode        Show current proving mode
    help        Show this help message
    version     Show version information

PROVING MODES:
    commitment-mode (default)  Fast but NOT zero-knowledge
    real-nova                  Real ZK proofs (slow)

BUILD FOR REAL ZK:
    cargo build --features real-nova --no-default-features
"#);
}

fn print_version() {
    println!("zk-origin-prover v{}", VERSION);
    println!("Proving mode: {}", proving_mode());
}

fn print_mode() {
    println!("Current proving mode: {}", proving_mode());
    println!("Is real ZK: {}", is_real_zk_enabled());
    
    let perf = expected_performance();
    println!("\nExpected performance:");
    println!("  Setup:        {}", perf.setup_time);
    println!("  Per step:     {}", perf.step_time);
    println!("  Compression:  {}", perf.compression_time);
    println!("  Verification: {}", perf.verification_time);
    println!("  Proof size:   {}", perf.proof_size);
}

fn run_demo() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════╗
║                    ZK-ORIGIN DEMO                             ║
║               Mode: {:40}║
╚═══════════════════════════════════════════════════════════════╝
"#, proving_mode());

    if !is_real_zk_enabled() {
        println!("  WARNING: Running in COMMITMENT MODE");
        println!("  This is NOT zero-knowledge!");
        println!("  For real ZK, rebuild with:");
        println!("    cargo build --features real-nova --no-default-features");
        println!();
    }

      println!(" Step 1: Creating Origin Policy");
    let policy = OriginPolicy::default();

      println!(
    "   Policy created with {} allowed transitions",
     policy.allowed_transitions().len()
            ); 

    // Step 2: Initialize prover
    println!("\n Step 2: Initializing Lineage Prover");
    
    if is_real_zk_enabled() {
        println!("    Setting up Nova parameters (this takes 30-120 seconds)...");
    }
    
    let start = Instant::now();
    
    let mut prover = match LineageProver::new(policy.clone()) {
        Ok(p) => p,
        Err(e) => {
            println!("    Failed to create prover: {}", e);
            return;
        }
    };
    
    let genesis = [0u8; 32];
    if let Err(e) = prover.initialize(genesis) {
        println!("    Failed to initialize: {}", e);
        return;
    }
    
    let init_time = start.elapsed();
    println!("    Prover initialized in {:?}", init_time);
    println!("   Genesis: 0x{}...", &hex::encode(&genesis)[..16]);

    // Step 3: Add transitions
    println!("\n Step 3: Adding Transitions");
    
    if is_real_zk_enabled() {
        println!("    Each step takes 500-2000ms with real Nova...");
    }
    
    let transitions = vec![
        (OriginClass::User, "Genesis → User"),
        (OriginClass::User, "User → User"),
        (OriginClass::User, "User → User"),
    ];

    let mut prev_state = genesis;
    for (i, (origin, desc)) in transitions.iter().enumerate() {
        let new_state = [(i + 1) as u8; 32];
        let transition = Transition::new(
            prev_state,
            new_state,
            *origin,
            (i as u64 + 1) * 1000,
        );
        
        let start = Instant::now();
        match prover.add_transition(transition) {
            Ok(_) => {
                let elapsed = start.elapsed();
                println!("    Step {}: {} ({:?})", i + 1, desc, elapsed);
            }
            Err(e) => {
                println!("    Step {}: {} - Error: {}", i + 1, desc, e);
                return;
            }
        }
        
        prev_state = new_state;
    }
    
    println!("   Current depth: {}", prover.current_depth());

    // Step 4: Generate proof
    println!("\n Step 4: Generating Proof");
    
    if is_real_zk_enabled() {
        println!("    Compressing proof (this takes 10-60 seconds)...");
    }
    
    let start = Instant::now();
    
    let proof = match prover.finalize() {
        Ok(p) => p,
        Err(e) => {
            println!("    Failed to generate proof: {}", e);
            return;
        }
    };
    
    let prove_time = start.elapsed();
    println!("    Proof generated in {:?}", prove_time);
    println!("   Proof size: {} bytes ({:.2} KB)", 
             proof.proof_size(), 
             proof.proof_size() as f64 / 1024.0);
    println!("   Is real ZK: {}", proof.is_real_zk());
    println!("   Depth: {} steps", proof.num_steps);
    println!("   Final lineage: 0x{}...", &proof.final_lineage.to_hex()[..16]);

    // Step 5: Verify
    println!("\n  Step 5: Verifying Proof");
    let start = Instant::now();
    
    match proof.verify() {
        Ok(true) => {
            let verify_time = start.elapsed();
            println!("    PROOF VALID ({:?})", verify_time);
        }
        Ok(false) => {
            println!("    PROOF INVALID");
        }
        Err(e) => {
            println!("    Verification error: {}", e);
        }
    }

    // Step 6: Policy enforcement demo
    println!("\n  Step 6: Testing Policy Enforcement");
    
    let mut test_prover = LineageProver::new(policy.clone()).unwrap();
    test_prover.initialize([0u8; 32]).unwrap();
    
    // Valid: Genesis -> User
    let valid = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
    match test_prover.validate_transition(&valid) {
        Ok(_) => println!("    Genesis → User: ALLOWED (correct)"),
        Err(e) => println!("    Genesis → User: BLOCKED - {}", e),
    }
    
    // Add the valid transition
    test_prover.add_transition(valid).unwrap();
    
    // Invalid: User -> Admin (not allowed by default policy)
    let invalid = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
    match test_prover.validate_transition(&invalid) {
        Ok(_) => println!("    User → Admin: ALLOWED (unexpected!)"),
        Err(_) => println!("    User → Admin: BLOCKED (correct - policy enforced)"),
    }

    // Summary
    println!("\n{}", "═".repeat(63));
    println!("                         SUMMARY");
    println!("{}", "═".repeat(63));
    println!("  Proving mode:     {}", proving_mode());
    println!("  Steps proven:     {}", proof.num_steps);
    println!("  Proof size:       {} bytes", proof.proof_size());
    println!("  Real ZK proof:    {}", proof.is_real_zk());
    println!("  Policy enforced:  ");
    
    if proof.is_real_zk() {
        println!("\n   This is a REAL zero-knowledge proof!");
        println!("  🔒 Cryptographically secure lineage verification.");
    } else {
        println!("\n    This is a COMMITMENT-based proof (not ZK).");
        println!("   Suitable for development and testing only.");
    }
    
    println!("{}", "═".repeat(63));
}

fn run_benchmark() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════╗
║                 ZK-ORIGIN BENCHMARKS                          ║
║               Mode: {:40}║
╚═══════════════════════════════════════════════════════════════╝
"#, proving_mode());

    if !is_real_zk_enabled() {
        println!("  Running in COMMITMENT MODE - these are NOT real ZK benchmarks!");
        println!("  Real Nova benchmarks will be 1000x slower.");
        println!();
    }

    let policy = OriginPolicy::default();

    // Benchmark 1: Initialization
    println!(" Benchmark 1: Prover Initialization");
    let iterations = if is_real_zk_enabled() { 1 } else { 10 };
    
    let start = Instant::now();
    for _ in 0..iterations {
        let mut prover = LineageProver::new(policy.clone()).unwrap();
        prover.initialize([0u8; 32]).unwrap();
    }
    let total = start.elapsed();
    let avg = total / iterations as u32;
    
    println!("   {} iterations: {:?}", iterations, total);
    println!("   Average: {:?}", avg);
    
    if is_real_zk_enabled() {
        println!("   (Nova setup is one-time cost, can be cached)");
    }

    // Benchmark 2: Adding transitions
    println!("\n Benchmark 2: Adding Transitions");
    
    let num_transitions = if is_real_zk_enabled() { 5 } else { 100 };
    
    let mut prover = LineageProver::new(policy.clone()).unwrap();
    prover.initialize([0u8; 32]).unwrap();
    
    let start = Instant::now();
    for i in 0..num_transitions {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(transition).unwrap();
    }
    let total = start.elapsed();
    let avg = total / num_transitions as u32;
    
    println!("   {} transitions: {:?}", num_transitions, total);
    println!("   Average per transition: {:?}", avg);
    
    if is_real_zk_enabled() {
        let tps = 1000.0 / avg.as_millis() as f64;
        println!("   Throughput: {:.2} transitions/sec", tps);
    } else {
        let tps = 1_000_000.0 / avg.as_micros() as f64;
        println!("   Throughput: {:.0} transitions/sec (commitment mode)", tps);
    }

    // Benchmark 3: Proof generation
    println!("\n Benchmark 3: Proof Generation (Finalization)");
    
    let start = Instant::now();
    let proof = prover.finalize().unwrap();
    let prove_time = start.elapsed();
    
    println!("   Depth {} proof: {:?}", proof.num_steps, prove_time);
    println!("   Proof size: {} bytes ({:.2} KB)", 
             proof.proof_size(),
             proof.proof_size() as f64 / 1024.0);
    println!("   Is real ZK: {}", proof.is_real_zk());

    // Benchmark 4: Verification
    println!("\n Benchmark 4: Proof Verification");
    
    let verify_iterations = if is_real_zk_enabled() { 1 } else { 100 };
    
    let start = Instant::now();
    for _ in 0..verify_iterations {
        let _ = proof.verify().unwrap();
    }
    let total = start.elapsed();
    let avg = total / verify_iterations as u32;
    
    println!("   {} verifications: {:?}", verify_iterations, total);
    println!("   Average: {:?}", avg);

    // Summary table
    println!("\n{}", "═".repeat(63));
    println!("                    BENCHMARK SUMMARY");
    println!("{}", "═".repeat(63));
    println!("  {:30} {:>15} {:>12}", "Operation", "Time", "Notes");
    println!("{}", "─".repeat(63));
    
    if is_real_zk_enabled() {
        println!("  {:30} {:>15} {:>12}", 
                 "Nova Setup", 
                 format!("{:?}", avg),
                 "one-time");
        println!("  {:30} {:>15} {:>12}", 
                 "Per Step", 
                 format!("{:?}", total / num_transitions as u32),
                 "real ZK");
        println!("  {:30} {:>15} {:>12}", 
                 "Compression", 
                 format!("{:?}", prove_time),
                 "real ZK");
    } else {
        println!("  {:30} {:>15} {:>12}", 
                 "Initialization", 
                 format!("{:?}", avg),
                 "fast");
        println!("  {:30} {:>15} {:>12}", 
                 "Per Transition", 
                 format!("{:?}", total / num_transitions as u32),
                 "NOT ZK");
        println!("  {:30} {:>15} {:>12}", 
                 "Finalization", 
                 format!("{:?}", prove_time),
                 "NOT ZK");
    }
    
    println!("  {:30} {:>15} {:>12}", 
             "Proof Size", 
             format!("{} B", proof.proof_size()),
             if proof.is_real_zk() { "real ZK" } else { "hash only" });
    
    println!("{}", "═".repeat(63));
    
    // Expected performance comparison
    println!("\n Expected Performance Comparison:");
    let perf = expected_performance();
    println!("  Mode: {}", proving_mode());
    println!("  Setup:        {}", perf.setup_time);
    println!("  Per Step:     {}", perf.step_time);
    println!("  Compression:  {}", perf.compression_time);
    println!("  Verification: {}", perf.verification_time);
    println!("  Proof Size:   {}", perf.proof_size);
    println!("  Real ZK:      {}", perf.is_real_zk);
}