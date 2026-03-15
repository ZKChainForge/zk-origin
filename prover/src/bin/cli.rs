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
    println!(
        r#"
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
"#
    );
}

fn print_version() {
    println!("zk-origin-prover v{}", VERSION);
    println!("Proving mode: {}", proving_mode());
}

fn print_mode() {
    println!("Current proving mode: {}", proving_mode());
    println!("Is real ZK: {}", is_real_zk_enabled());

    println!("\nExpected performance:");
    if is_real_zk_enabled() {
        println!("  Setup:        30-120 seconds (one-time)");
        println!("  Per step:     500-2000 ms");
        println!("  Compression:  10-60 seconds");
        println!("  Verification: 5-20 ms");
        println!("  Proof size:   ~10-50 KB");
    } else {
        println!("  Setup:        <1 ms");
        println!("  Per step:     10-50 µs");
        println!("  Compression:  <1 ms");
        println!("  Verification: <1 ms");
        println!("  Proof size:   ~100 bytes");
    }
}

fn run_demo() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║                    ZK-ORIGIN DEMO                             ║
║               Mode: {:40}                                     ║
╚═══════════════════════════════════════════════════════════════╝
"#,
        proving_mode()
    );

    if !is_real_zk_enabled() {
        println!("   WARNING: Running in COMMITMENT MODE");
        println!("  This is NOT zero-knowledge!");
        println!("  For real ZK, rebuild with:");
        println!("    cargo build --features real-nova --no-default-features");
        println!();
    }

    println!("═ Step 1: Creating Origin Policy");
    let policy = OriginPolicy::default();

    println!(
        "   Policy created with {} allowed transitions",
        policy.allowed_transitions().len()
    );

    // Step 2: Initialize prover
    println!("\n═ Step 2: Initializing Lineage Prover");

    if is_real_zk_enabled() {
        println!("   Setting up Nova parameters (this takes 30-120 seconds)...");
    }

    let start = Instant::now();

    let mut prover = create_prover(&policy);

    let genesis = [0u8; 32];
    if let Err(e) = prover.initialize(genesis) {
        println!("   Failed to initialize: {}", e);
        return;
    }

    let init_time = start.elapsed();
    println!("   Prover initialized in {:?}", init_time);
    println!("  Genesis: 0x{}...", hex_encode(&genesis, 16));

    // Step 3: Add transitions
    println!("\n═ Step 3: Adding Transitions");

    if is_real_zk_enabled() {
        println!("   Each step takes 500-2000ms with real Nova...");
    }

    let transitions = [
        (OriginClass::User, "Genesis → User"),
        (OriginClass::User, "User → User"),
        (OriginClass::User, "User → User"),
    ];

    let mut prev_state = genesis;
    for (i, (origin, desc)) in transitions.iter().enumerate() {
        let new_state = [(i + 1) as u8; 32];
        let transition = Transition::new(prev_state, new_state, *origin, (i as u64 + 1) * 1000);

        let start = Instant::now();
        match prover.add_transition(transition) {
            Ok(_) => {
                let elapsed = start.elapsed();
                println!("   Step {}: {} ({:?})", i + 1, desc, elapsed);
            }
            Err(e) => {
                println!("   Step {}: {} - Error: {}", i + 1, desc, e);
                return;
            }
        }

        prev_state = new_state;
    }

    println!("  Current depth: {}", prover.current_depth());

    // Step 4: Generate proof
    println!("\n═ Step 4: Generating Proof");

    if is_real_zk_enabled() {
        println!("   Compressing proof (this takes 10-60 seconds)...");
    }

    let start = Instant::now();

    let proof = match prover.finalize() {
        Ok(p) => p,
        Err(e) => {
            println!("   Failed to generate proof: {}", e);
            return;
        }
    };

    let prove_time = start.elapsed();
    println!("   Proof generated in {:?}", prove_time);
    println!(
        "  Proof size: {} bytes ({:.2} KB)",
        proof.proof_size(),
        proof.proof_size() as f64 / 1024.0
    );
    println!("  Is real ZK: {}", proof.is_real_zk());
    println!("  Depth: {} steps", proof.num_steps);
    println!(
        "  Final lineage: 0x{}...",
        &proof.final_lineage.to_hex()[..16.min(proof.final_lineage.to_hex().len())]
    );

    // Step 5: Verify proof
    println!("\n═ Step 5: Verifying Proof");

    let start = Instant::now();

    // Use the genesis commitment from the proof itself for consistency
    let verifier = LineageVerifier::from_proof(&proof, &policy);

    match verifier.verify(&proof) {
        Ok(true) => {
            let verify_time = start.elapsed();
            println!("    Structural verification passed ({:?})", verify_time);

            if proof.is_real_zk() {
                // For real ZK, also do cryptographic verification
                #[cfg(feature = "real-nova")]
                {
                    println!("   Performing cryptographic ZK verification...");
                    let zk_start = Instant::now();

                    match verifier.verify_zk(&proof) {
                        Ok(true) => {
                            println!(
                                "    CRYPTOGRAPHIC ZK VERIFIED ({:?})",
                                zk_start.elapsed()
                            );
                            println!("     Proof size: {} bytes", proof.proof_size());
                            println!("     Depth: {} steps", proof.num_steps);
                        }
                        Ok(false) => println!("    Cryptographic verification returned false"),
                        Err(e) => println!("    ZK verification error: {}", e),
                    }
                }

                #[cfg(not(feature = "real-nova"))]
                {
                    println!("    VERIFIED ({:?})", verify_time);
                    println!("     Proof size: {} bytes", proof.proof_size());
                    println!("     Depth: {} steps", proof.num_steps);
                }
            } else {
                println!("    STRUCTURAL CHECK PASSED ({:?})", verify_time);
                println!("     (Not cryptographic - rebuild with real-nova for ZK)");
            }
        }
        Ok(false) => {
            println!("    Verification failed");
        }
        Err(e) => {
            println!("    Verification error: {}", e);
        }
    }

    // Step 6: Policy enforcement demo
    println!("\n═ Step 6: Testing Policy Enforcement");

    let mut test_prover = create_prover(&policy);
    let _ = test_prover.initialize([0u8; 32]);

    // Valid: Genesis -> User
    let valid = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
    match test_prover.validate_transition(&valid) {
        Ok(_) => {
            let _ = test_prover.add_transition(valid);
            println!("   Genesis → User: ALLOWED (correct)");
        }
        Err(e) => println!("   Genesis → User: BLOCKED - {}", e),
    }

    // Invalid: User -> Admin (not allowed by default policy)
    let invalid = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
    match test_prover.validate_transition(&invalid) {
        Ok(_) => println!("   User → Admin: ALLOWED (unexpected!)"),
        Err(_) => println!("   User → Admin: BLOCKED (correct - policy enforced)"),
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
        println!("  Cryptographically secure lineage verification.");
    } else {
        println!("\n   This is a COMMITMENT-based proof (not ZK).");
        println!("  Suitable for development and testing only.");
    }

    println!("{}", "═".repeat(63));
}

fn run_benchmark() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║                 ZK-ORIGIN BENCHMARKS                          ║
║               Mode: {:40}                                     ║
╚═══════════════════════════════════════════════════════════════╝
"#,
        proving_mode()
    );

    if !is_real_zk_enabled() {
        println!("   WARNING: Running in COMMITMENT MODE - these are NOT real ZK benchmarks!");
        println!();
        println!("  To run REAL ZK benchmarks:");
        println!("    cargo build --release --features real-nova --no-default-features");
        println!("    ./target/release/zk-origin-cli benchmark");
        println!();
    }

    let policy = OriginPolicy::default();

    // Benchmark 1: Initialization
    println!("═ Benchmark 1: Prover Initialization");
    let iterations = if is_real_zk_enabled() { 1 } else { 10 };

    let start = Instant::now();
    for _ in 0..iterations {
        let mut prover = create_prover(&policy);
        let _ = prover.initialize([0u8; 32]);
    }
    let total = start.elapsed();
    let avg = total / iterations as u32;

    println!("  {} iterations: {:?}", iterations, total);
    println!("  Average: {:?}", avg);

    if is_real_zk_enabled() {
        println!("  (Nova setup is one-time cost, can be cached)");
    }

    // Benchmark 2: Adding transitions
    println!("\n═ Benchmark 2: Adding Transitions");

    let num_transitions = if is_real_zk_enabled() { 5 } else { 100 };

    let mut prover = create_prover(&policy);
    let _ = prover.initialize([0u8; 32]);

    let start = Instant::now();
    for i in 0..num_transitions {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        let _ = prover.add_transition(transition);
    }
    let total = start.elapsed();
    let avg = total / num_transitions as u32;

    println!("  {} transitions: {:?}", num_transitions, total);
    println!("  Average per transition: {:?}", avg);

    if is_real_zk_enabled() {
        let tps = 1000.0 / avg.as_millis().max(1) as f64;
        println!("  Throughput: {:.2} transitions/sec", tps);
    } else {
        let micros = avg.as_micros().max(1);
        let tps = 1_000_000.0 / micros as f64;
        println!("  Throughput: {:.0} transitions/sec (commitment mode)", tps);
    }

    // Benchmark 3: Proof generation
    println!("\n═ Benchmark 3: Proof Generation (Finalization)");

    let start = Instant::now();
    let proof = match prover.finalize() {
        Ok(p) => p,
        Err(e) => {
            println!("  Failed to finalize: {}", e);
            return;
        }
    };
    let prove_time = start.elapsed();

    println!("  Depth {} proof: {:?}", proof.num_steps, prove_time);
    println!(
        "  Proof size: {} bytes ({:.2} KB)",
        proof.proof_size(),
        proof.proof_size() as f64 / 1024.0
    );
    println!("  Is real ZK: {}", proof.is_real_zk());

    // Benchmark 4: Verification
    println!("\n═ Benchmark 4: Proof Verification");

    let verify_iterations = if is_real_zk_enabled() { 1 } else { 100 };

    let start = Instant::now();
    for _ in 0..verify_iterations {
        let verifier = LineageVerifier::from_proof(&proof, &policy);
        let _ = verifier.verify(&proof);
    }
    let total = start.elapsed();
    let avg = total / verify_iterations as u32;

    println!("  {} verifications: {:?}", verify_iterations, total);
    println!("  Average: {:?}", avg);

    // ZK Verification benchmark (if real-nova)
    #[cfg(feature = "real-nova")]
    if proof.is_real_zk() {
        println!("\n═ Benchmark 5: ZK Verification");

        let verifier = LineageVerifier::from_proof(&proof, &policy);
        let start = Instant::now();
        match verifier.verify_zk(&proof) {
            Ok(true) => {
                let zk_time = start.elapsed();
                println!("  ZK verification: {:?}", zk_time);
            }
            Ok(false) => println!("  ZK verification returned false"),
            Err(e) => println!("  ZK verification error: {}", e),
        }
    }

    // Summary table
    println!("\n{}", "═".repeat(63));
    println!("                    BENCHMARK SUMMARY");
    println!("{}", "═".repeat(63));
    println!("  {:30} {:>15} {:>12}", "Operation", "Time", "Notes");
    println!("{}", "─".repeat(63));

    if is_real_zk_enabled() {
        println!(
            "  {:30} {:>15} {:>12}",
            "Nova Setup",
            format!("{:?}", avg),
            "one-time"
        );
        println!(
            "  {:30} {:>15} {:>12}",
            "Per Step",
            format!("{:?}", total / num_transitions as u32),
            "real ZK"
        );
        println!(
            "  {:30} {:>15} {:>12}",
            "Compression",
            format!("{:?}", prove_time),
            "real ZK"
        );
    } else {
        println!(
            "  {:30} {:>15} {:>12}",
            "Initialization",
            format!("{:?}", avg),
            "fast"
        );
        println!(
            "  {:30} {:>15} {:>12}",
            "Per Transition",
            format!("{:?}", total / num_transitions as u32),
            "NOT ZK"
        );
        println!(
            "  {:30} {:>15} {:>12}",
            "Finalization",
            format!("{:?}", prove_time),
            "NOT ZK"
        );
    }

    println!(
        "  {:30} {:>15} {:>12}",
        "Proof Size",
        format!("{} B", proof.proof_size()),
        if proof.is_real_zk() {
            "real ZK"
        } else {
            "hash only"
        }
    );

    println!("{}", "═".repeat(63));

   
}

/// Helper function to create a prover based on the active feature
#[cfg(feature = "real-nova")]
fn create_prover(policy: &OriginPolicy) -> LineageProver<'static> {
    let params = get_static_nova_params();
    LineageProver::new(policy.clone(), params).unwrap()
}

/// Get static Nova params (cached, one-time setup)
#[cfg(feature = "real-nova")]
fn get_static_nova_params() -> &'static NovaParams {
    use std::sync::OnceLock;

    static PARAMS: OnceLock<NovaParams> = OnceLock::new();

    PARAMS.get_or_init(|| {
        println!("   Setting up Nova params (one-time cost)...");
        LineageProver::setup_params(&OriginPolicy::default()).unwrap()
    })
}

#[cfg(not(feature = "real-nova"))]
fn create_prover(policy: &OriginPolicy) -> LineageProver<'static> {
    LineageProver::new(policy.clone()).unwrap()
}

/// Helper to encode hex (first n bytes)
fn hex_encode(bytes: &[u8], len: usize) -> String {
    bytes
        .iter()
        .take(len / 2)
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}