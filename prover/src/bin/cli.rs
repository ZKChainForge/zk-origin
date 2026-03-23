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
    real-nova                  Nova IVC proofs (~10KB)
    compact-zk                 Groth16 proofs (<1KB) ⭐ NEW

BUILD COMMANDS:
    # Fast mode (not ZK) - Default
    cargo build --release

    # Nova IVC (~10KB proofs)
    cargo build --release --features real-nova --no-default-features

    # Compact Groth16 (<1KB proofs) 
    cargo build --release --features compact-zk --no-default-features
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
    
    #[cfg(feature = "compact-zk")]
    println!("Is compact ZK: {}", is_compact_zk_enabled());
    
    #[cfg(feature = "real-nova")]
    println!("Is Nova: {}", is_nova_enabled());

    println!("\nExpected performance:");
    
    #[cfg(feature = "compact-zk")]
    {
        println!("  Setup:        5-30 seconds (trusted setup)");
        println!("  Witness:      <1 ms per step");
        println!("  Prove:        1-10 seconds (all at once)");
        println!("  Verification: <50 ms");
        println!("  Proof size:   192 bytes ⭐");
    }
    
    #[cfg(all(feature = "real-nova", not(feature = "compact-zk")))]
    {
        println!("  Setup:        30-120 seconds (one-time)");
        println!("  Per step:     40-125 ms");
        println!("  Compression:  1-2 seconds");
        println!("  Verification: ~100 ms");
        println!("  Proof size:   ~10 KB");
    }
    
    #[cfg(all(feature = "commitment-mode", not(feature = "real-nova"), not(feature = "compact-zk")))]
    {
        println!("  Setup:        <1 ms");
        println!("  Per step:     10-50 µs");
        println!("  Finalization: <1 ms");
        println!("  Verification: <1 ms");
        println!("  Proof size:   ~100 bytes (NOT ZK)");
    }
}

fn run_demo() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║                    ZK-ORIGIN DEMO                             ║
║               Mode: {:40}║
╚═══════════════════════════════════════════════════════════════╝
"#,
        proving_mode()
    );

    #[cfg(feature = "compact-zk")]
    run_demo_compact();

    #[cfg(all(feature = "real-nova", not(feature = "compact-zk")))]
    run_demo_nova();

    #[cfg(all(feature = "commitment-mode", not(feature = "real-nova"), not(feature = "compact-zk")))]
    run_demo_commitment();
}

// ============================================================================
// COMPACT ZK (GROTH16) DEMO
// ============================================================================

#[cfg(feature = "compact-zk")]
fn run_demo_compact() {
    println!("═ Step 1: Creating Origin Policy");
    let policy = OriginPolicy::default();
    println!(
        "   Policy created with {} allowed transitions",
        policy.allowed_transitions().len()
    );

    println!("\n═ Step 2: Setting up Groth16 (Trusted Setup)");
    println!("   This creates proving/verifying keys...");
    let start = Instant::now();

    let params = match Groth16Params::setup(policy.compute_hash()) {
        Ok(p) => p,
        Err(e) => {
            println!("   Failed to setup: {}", e);
            return;
        }
    };

    println!("   Setup complete in {:?}", start.elapsed());

    println!("\n═ Step 3: Initializing Prover");
    let mut prover = Groth16LineageProver::new(&params);
    
    // Use raw genesis values (zeros)
    let genesis_lineage = [0u8; 32];
    let genesis_counters = [0u8; 32];

    if let Err(e) = prover.initialize(genesis_lineage, genesis_counters) {
        println!("   Failed to initialize: {}", e);
        return;
    }
    println!("   Prover initialized");
    println!("  Genesis: 0x{}...", hex_encode(&genesis_lineage, 16));

    println!("\n═ Step 4: Adding Transitions (Witness Collection)");
    let transitions_data = [
        (OriginClass::User, "Genesis → User"),
        (OriginClass::User, "User → User"),
        (OriginClass::User, "User → User"),
    ];

    let mut prev_state = [0u8; 32];
    for (i, (origin, desc)) in transitions_data.iter().enumerate() {
        let mut new_state = [0u8; 32];
        new_state[0] = (i + 1) as u8;

        let witness = StepWitness {
            prev_state_hash: prev_state,
            new_state_hash: new_state,
            prev_lineage_commitment: [0u8; 32],
            prev_origin: if i == 0 {
                OriginClass::Genesis
            } else {
                OriginClass::User
            },
            prev_depth: i as u64,
            new_origin: *origin,
            timestamp: (i as u64 + 1) * 1000,
            policy_proof: vec![],
            policy_indices: vec![],
            policy_root: policy.compute_hash(),
            epoch_id: 0,
            prev_counters: [0; 6],
            rate_limits: [1, u32::MAX, 10, 100, 5, 1000],
            prev_counter_commitment: [0u8; 32],
        };

        match prover.prove_step(&witness) {
            Ok(_) => println!("   Step {}: {}", i + 1, desc),
            Err(e) => {
                println!("   Step {}: {} - Error: {}", i + 1, desc, e);
                return;
            }
        }

        prev_state = new_state;
    }

    println!("\n═ Step 5: Generating Compact Proof");
    println!("   Creating Groth16 proof (all transitions at once)...");
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
        "  ⭐ Proof size: {} bytes (< 1KB!)",
        proof.proof_size()
    );
    println!("  Is real ZK: true");
    println!("  Depth: {} steps", proof.num_steps);
    println!(
        "  Final lineage: 0x{}...",
        &proof.final_lineage.to_hex()[..16.min(proof.final_lineage.to_hex().len())]
    );

    println!("\n═ Step 6: Verifying Proof");
    let start = Instant::now();

    // Use the values from the proof
    match verify_groth16_proof(
        &proof.proof_bytes,
        proof.verifier_key.as_ref().unwrap(),
        &genesis_lineage,  // Use the same genesis we initialized with
        &genesis_counters, // Use the same counters we initialized with
        &proof.final_lineage.value,
        &proof.final_counters.value,
    ) {
        Ok(true) => {
            println!("    CRYPTOGRAPHIC ZK VERIFIED ({:?})", start.elapsed());
            println!("     Proof size: {} bytes", proof.proof_size());
            println!("     Depth: {} steps", proof.num_steps);
        }
        Ok(false) => {
            println!("    Verification returned false");
        }
        Err(e) => {
            println!("    Verification error: {}", e);
        }
    }

    
}
// ============================================================================
// NOVA IVC DEMO
// ============================================================================

#[cfg(all(feature = "real-nova", not(feature = "compact-zk")))]
fn run_demo_nova() {
    println!("═ Step 1: Creating Origin Policy");
    let policy = OriginPolicy::default();
    println!(
        "   Policy created with {} allowed transitions",
        policy.allowed_transitions().len()
    );

    println!("\n═ Step 2: Initializing Lineage Prover");
    println!("   Setting up Nova parameters (this takes 30-120 seconds)...");

    let start = Instant::now();
    let mut prover = create_prover_nova(&policy);
    let genesis = [0u8; 32];

    if let Err(e) = prover.initialize(genesis) {
        println!("   Failed to initialize: {}", e);
        return;
    }

    println!("   Prover initialized in {:?}", start.elapsed());
    println!("  Genesis: 0x{}...", hex_encode(&genesis, 16));

    println!("\n═ Step 3: Adding Transitions");
    println!("   Each step takes 40-125ms with real Nova...");

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
            Ok(_) => println!("   Step {}: {} ({:?})", i + 1, desc, start.elapsed()),
            Err(e) => {
                println!("   Step {}: {} - Error: {}", i + 1, desc, e);
                return;
            }
        }
        prev_state = new_state;
    }
    println!("  Current depth: {}", prover.current_depth());

    println!("\n═ Step 4: Generating Proof");
    println!("   Compressing proof (this takes 1-2 seconds)...");

    let start = Instant::now();
    let proof = match prover.finalize() {
        Ok(p) => p,
        Err(e) => {
            println!("   Failed to generate proof: {}", e);
            return;
        }
    };

    println!("   Proof generated in {:?}", start.elapsed());
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

    println!("\n═ Step 5: Verifying Proof");
    let start = Instant::now();
    let verifier = LineageVerifier::from_proof(&proof, &policy);

    match verifier.verify(&proof) {
        Ok(true) => {
            println!("    Structural verification passed ({:?})", start.elapsed());

            if proof.is_real_zk() {
                println!("   Performing cryptographic ZK verification...");
                let zk_start = Instant::now();

                match verifier.verify_zk(&proof) {
                    Ok(true) => {
                        println!("    CRYPTOGRAPHIC ZK VERIFIED ({:?})", zk_start.elapsed());
                        println!("     Proof size: {} bytes", proof.proof_size());
                        println!("     Depth: {} steps", proof.num_steps);
                    }
                    Ok(false) => println!("    Cryptographic verification returned false"),
                    Err(e) => println!("    ZK verification error: {}", e),
                }
            }
        }
        Ok(false) => println!("    Verification failed"),
        Err(e) => println!("    Verification error: {}", e),
    }

    // Policy enforcement test
    println!("\n═ Step 6: Testing Policy Enforcement");
    let mut test_prover = create_prover_nova(&policy);
    let _ = test_prover.initialize([0u8; 32]);

    let valid = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
    match test_prover.validate_transition(&valid) {
        Ok(_) => {
            let _ = test_prover.add_transition(valid);
            println!("   Genesis → User: ALLOWED (correct)");
        }
        Err(e) => println!("   Genesis → User: BLOCKED - {}", e),
    }

    let invalid = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
    match test_prover.validate_transition(&invalid) {
        Ok(_) => println!("   User → Admin: ALLOWED (unexpected!)"),
        Err(_) => println!("   User → Admin: BLOCKED (correct - policy enforced)"),
    }

  
}

// ============================================================================
// COMMITMENT MODE DEMO
// ============================================================================

#[cfg(all(feature = "commitment-mode", not(feature = "real-nova"), not(feature = "compact-zk")))]
fn run_demo_commitment() {
    println!("   WARNING: Running in COMMITMENT MODE");
    println!("  This is NOT zero-knowledge!");
    println!("  For real ZK, rebuild with:");
    println!("    cargo build --features real-nova --no-default-features");
    println!("  Or for compact ZK (<1KB):");
    println!("    cargo build --features compact-zk --no-default-features");
    println!();

    println!("═ Step 1: Creating Origin Policy");
    let policy = OriginPolicy::default();
    println!(
        "   Policy created with {} allowed transitions",
        policy.allowed_transitions().len()
    );

    println!("\n═ Step 2: Initializing Prover");
    let mut prover = create_prover_commitment(&policy);
    let genesis = [0u8; 32];
    let _ = prover.initialize(genesis);
    println!("   Prover initialized");
    println!("  Genesis: 0x{}...", hex_encode(&genesis, 16));

    println!("\n═ Step 3: Adding Transitions");
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
            Ok(_) => println!("   Step {}: {} ({:?})", i + 1, desc, start.elapsed()),
            Err(e) => {
                println!("   Step {}: {} - Error: {}", i + 1, desc, e);
                return;
            }
        }
        prev_state = new_state;
    }

    println!("\n═ Step 4: Generating Proof");
    let start = Instant::now();
    let proof = match prover.finalize() {
        Ok(p) => p,
        Err(e) => {
            println!("   Failed: {}", e);
            return;
        }
    };
    println!("   Proof generated in {:?}", start.elapsed());
    println!("  Proof size: {} bytes", proof.proof_size());

    println!("\n═ Step 5: Verifying Proof");
    let start = Instant::now();
    let verifier = LineageVerifier::from_proof(&proof, &policy);
    match verifier.verify(&proof) {
        Ok(true) => println!("    Structural check passed ({:?})", start.elapsed()),
        Ok(false) => println!("    Verification failed"),
        Err(e) => println!("    Error: {}", e),
    }

   
}

// ============================================================================
// BENCHMARKS
// ============================================================================

fn run_benchmark() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║                 ZK-ORIGIN BENCHMARKS                          ║
║               Mode: {:40}║
╚═══════════════════════════════════════════════════════════════╝
"#,
        proving_mode()
    );

    #[cfg(feature = "compact-zk")]
    run_benchmark_compact();

    #[cfg(all(feature = "real-nova", not(feature = "compact-zk")))]
    run_benchmark_nova();

    #[cfg(all(feature = "commitment-mode", not(feature = "real-nova"), not(feature = "compact-zk")))]
    run_benchmark_commitment();
}

#[cfg(feature = "compact-zk")]
fn run_benchmark_compact() {
    let policy = OriginPolicy::default();

    println!("═ Benchmark 1: Groth16 Setup");
    let start = Instant::now();
    let params = Groth16Params::setup(policy.compute_hash()).unwrap();
    println!("  Setup time: {:?}", start.elapsed());

    println!("\n═ Benchmark 2: Proof Generation (3 transitions)");
    let mut prover = Groth16LineageProver::new(&params);
    prover.initialize([0u8; 32], [0u8; 32]).unwrap();

    for i in 0..3 {
        let witness = StepWitness {
            prev_state_hash: [i as u8; 32],
            new_state_hash: [(i + 1) as u8; 32],
            prev_lineage_commitment: [0u8; 32],
            prev_origin: OriginClass::User,
            prev_depth: i as u64,
            new_origin: OriginClass::User,
            timestamp: (i as u64 + 1) * 1000,
            policy_proof: vec![],
            policy_indices: vec![],
            policy_root: policy.compute_hash(),
            epoch_id: 0,
            prev_counters: [0; 6],
            rate_limits: [1, u32::MAX, 10, 100, 5, 1000],
            prev_counter_commitment: [0u8; 32],
        };
        prover.prove_step(&witness).unwrap();
    }

    let start = Instant::now();
    let proof = prover.finalize().unwrap();
    println!("  Prove time: {:?}", start.elapsed());
    println!("  Proof size: {} bytes", proof.proof_size());

    println!("\n═ Benchmark 3: Verification");
    let start = Instant::now();
    let _ = verify_groth16_proof(
        &proof.proof_bytes,
        proof.verifier_key.as_ref().unwrap(),
        &proof.genesis_commitment.value,
        &proof.initial_counter_commitment,
        &proof.final_lineage.value,
        &proof.final_counters.value,
    );
    println!("  Verify time: {:?}", start.elapsed());

    println!("\n{}", "═".repeat(50));
    println!("  Mode: Compact ZK (Groth16)");
    println!("  Proof size: {} bytes ⭐", proof.proof_size());
    println!("{}", "═".repeat(50));
}

#[cfg(all(feature = "real-nova", not(feature = "compact-zk")))]
fn run_benchmark_nova() {
    let policy = OriginPolicy::default();

    println!("═ Benchmark 1: Nova Setup");
    let start = Instant::now();
    let mut prover = create_prover_nova(&policy);
    prover.initialize([0u8; 32]).unwrap();
    println!("  Setup time: {:?}", start.elapsed());

    println!("\n═ Benchmark 2: Adding 5 Transitions");
    let start = Instant::now();
    for i in 0..5 {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(transition).unwrap();
    }
    let total = start.elapsed();
    println!("  Total: {:?}", total);
    println!("  Per step: {:?}", total / 5);

    println!("\n═ Benchmark 3: Proof Generation");
    let start = Instant::now();
    let proof = prover.finalize().unwrap();
    println!("  Compression time: {:?}", start.elapsed());
    println!("  Proof size: {} bytes", proof.proof_size());

    println!("\n═ Benchmark 4: Verification");
    let verifier = LineageVerifier::from_proof(&proof, &policy);
    let start = Instant::now();
    let _ = verifier.verify_zk(&proof);
    println!("  Verify time: {:?}", start.elapsed());

    println!("\n{}", "═".repeat(50));
    println!("  Mode: Nova IVC");
    println!("  Proof size: {} bytes", proof.proof_size());
    println!("{}", "═".repeat(50));
}

#[cfg(all(feature = "commitment-mode", not(feature = "real-nova"), not(feature = "compact-zk")))]
fn run_benchmark_commitment() {
    let policy = OriginPolicy::default();

    println!("═ Benchmark 1: Setup");
    let start = Instant::now();
    let mut prover = create_prover_commitment(&policy);
    prover.initialize([0u8; 32]).unwrap();
    println!("  Setup time: {:?}", start.elapsed());

    println!("\n═ Benchmark 2: Adding 100 Transitions");
    let start = Instant::now();
    for i in 0..100 {
        let transition = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover.add_transition(transition).unwrap();
    }
    let total = start.elapsed();
    println!("  Total: {:?}", total);
    println!("  Per step: {:?}", total / 100);

    println!("\n═ Benchmark 3: Proof Generation");
    let start = Instant::now();
    let proof = prover.finalize().unwrap();
    println!("  Finalize time: {:?}", start.elapsed());
    println!("  Proof size: {} bytes", proof.proof_size());

    println!("\n{}", "═".repeat(50));
    println!("  Mode: Commitment (NOT ZK)");
    println!("  Proof size: {} bytes", proof.proof_size());
    println!("{}", "═".repeat(50));
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

#[cfg(feature = "real-nova")]
fn create_prover_nova(policy: &OriginPolicy) -> LineageProver<'static> {
    let params = get_static_nova_params();
    LineageProver::new(policy.clone(), params).unwrap()
}

#[cfg(feature = "real-nova")]
fn get_static_nova_params() -> &'static NovaParams {
    use std::sync::OnceLock;

    static PARAMS: OnceLock<NovaParams> = OnceLock::new();

    PARAMS.get_or_init(|| {
        println!("   Setting up Nova params (one-time cost)...");
        LineageProver::setup_params(&OriginPolicy::default()).unwrap()
    })
}

#[cfg(all(feature = "commitment-mode", not(feature = "real-nova"), not(feature = "compact-zk")))]
fn create_prover_commitment(policy: &OriginPolicy) -> LineageProver<'static> {
    LineageProver::new(policy.clone()).unwrap()
}

fn hex_encode(bytes: &[u8], len: usize) -> String {
    bytes
        .iter()
        .take(len / 2)
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}