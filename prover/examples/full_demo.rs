//! Full demo with attack simulation

use zk_origin::{
    LineageProver,
    OriginClass,
    Transition,
    OriginPolicy,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" ZK-ORIGIN Full Demo");
    println!("{}", "=".repeat(50));

    // -----------------------------------
    // Setup policy & prover
    // -----------------------------------
    let policy = OriginPolicy::default();
    let mut prover = LineageProver::new(policy)?;

    // -----------------------------------
    // Part 1: Valid flow
    // -----------------------------------
    println!("\n Part 1: Valid Protocol Lifecycle\n");

    let genesis_state = [0u8; 32];

    let valid_transitions = vec![
        (OriginClass::Admin, "Protocol initialization"),
        (OriginClass::User, "First user deposit"),
        (OriginClass::User, "User swap"),
        (OriginClass::User, "Another deposit"),
    ];

    let mut prev_state = genesis_state;
    let mut prev_origin = OriginClass::Genesis;

    for (i, (origin, description)) in valid_transitions.iter().enumerate() {
        let new_state = [(i + 1) as u8; 32];

        println!("  Step {}: {:?} -> {:?}", i + 1, prev_origin, origin);
        println!("          {}", description);

        prover.add_transition(Transition::new(
            prev_state,
            new_state,
            *origin,
            (i + 1) as u64 * 1_000,
        ))?;

        prev_state = new_state;
        prev_origin = *origin;
    }

    println!("\n   All valid transitions completed!");
    println!("  Final lineage: {:?}", prover.current_lineage());

    // -----------------------------------
    // Part 2: Attack simulation
    // -----------------------------------
    println!("\n{}", "=".repeat(50));
    println!("\n Part 2: Attack Simulation\n");

    println!("  Attempting privilege escalation (User -> Admin)...");

    let attack_result = prover.add_transition(Transition::new(
        prev_state,
        [99u8; 32],
        OriginClass::Admin, // ATTACK
        9_999,
    ));

    match attack_result {
        Ok(_) => println!("   ❌ Attack succeeded (BUG)"),
        Err(e) => println!("   ✅ Attack blocked: {}", e),
    }

    // -----------------------------------
    // Part 3: Valid admin flow
    // -----------------------------------
    println!("\n{}", "=".repeat(50));
    println!("\n Part 3: Proper Admin Flow\n");

    let admin_policy = OriginPolicy::default();
    let mut admin_prover = LineageProver::new(admin_policy)?;

    let admin_flow = vec![
        (OriginClass::Admin, "Initial admin setup"),
        (OriginClass::Admin, "Configure parameters"),
        (OriginClass::Bridge, "Enable bridge"),
        (OriginClass::User, "Bridge deposit arrives"),
    ];

    let mut prev_state = [100u8; 32];
    let mut prev_origin = OriginClass::Genesis;

    for (i, (origin, description)) in admin_flow.iter().enumerate() {
        let new_state = [(101 + i) as u8; 32];

        println!("  Step {}: {:?} -> {:?}", i + 1, prev_origin, origin);
        println!("          {}", description);

        admin_prover.add_transition(Transition::new(
            prev_state,
            new_state,
            *origin,
            (i + 1) as u64 * 1_000,
        ))?;

        prev_state = new_state;
        prev_origin = *origin;
    }

    println!("\n   Admin flow completed!");
    println!("  Final lineage: {:?}", admin_prover.current_lineage());

    // -----------------------------------
    // Proof generation
    // -----------------------------------
    println!("\n{}", "=".repeat(50));
    println!("\n Generating Final Proofs\n");

    let proof1 = prover.finalize()?;
    let proof2 = admin_prover.finalize()?;

    let valid1 = proof1.verify()?;
    let valid2 = proof2.verify()?;

    println!(
        "  User flow proof: {} bytes, valid: {}",
        proof1.proof_bytes.len(),
        valid1
    );

    println!(
        "  Admin flow proof: {} bytes, valid: {}",
        proof2.proof_bytes.len(),
        valid2
    );

    println!("\n Demo complete!");

    Ok(())
}