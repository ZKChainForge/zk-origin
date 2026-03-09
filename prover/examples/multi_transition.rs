//! Example demonstrating multiple transitions with different origin classes

use std::time::Instant;
use zk_origin::{LineageProver, OriginClass, OriginPolicy, Result, Transition, ZkOriginError};

fn main() -> Result<()> {
    println!("ZK-ORIGIN Multi-Transition \n");

    // Create prover with default policy
    let policy = OriginPolicy::default();
    let mut prover = LineageProver::new(policy.clone())?;
    prover.initialize([0u8; 32])?;

    println!("Testing various transition scenarios:\n");

    // Scenario 1: Valid sequence
    println!(" Scenario 1: Valid transition sequence ");
    let valid_sequence = vec![
        (OriginClass::User, "Genesis → User"),
        (OriginClass::User, "User → User"),
        (OriginClass::User, "User → User"),
    ];

    for (i, (origin, desc)) in valid_sequence.iter().enumerate() {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            *origin,
            (i as u64 + 1) * 1000,
        );

        match prover.add_transition(t) {
            Ok(_) => println!("   {}", desc),
            Err(e) => println!("  ✗ {}: {}", desc, e),
        }
    }

    // Generate proof for valid sequence
    let proof1 = prover.finalize()?;
    println!("  Generated proof with {} steps\n", proof1.num_steps);

    // Scenario 2: Policy violation
    println!(" Scenario 2: Policy violation ");
    let mut prover2 = LineageProver::new(policy.clone())?;
    prover2.initialize([0u8; 32])?;

    // Genesis → User (valid)
    prover2.add_transition(Transition::new(
        [0u8; 32],
        [1u8; 32],
        OriginClass::User,
        1000,
    ))?;
    println!("   Genesis → User");

    // User → Admin (INVALID - not allowed in default policy)
    let invalid_transition = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);

    match prover2.add_transition(invalid_transition) {
        Ok(_) => println!("   User → Admin (unexpected!)"),
        Err(ZkOriginError::PolicyViolation { from, to }) => {
            println!(
                "   User → Admin: Policy violation ({} → {} not allowed)",
                from, to
            );
        }
        Err(e) => println!("   User → Admin: {}", e),
    }

    // Scenario 3: Admin transitions (with rate limiting)
    println!("\n Scenario 3: Admin transitions with rate limiting ");
    let mut prover3 = LineageProver::new(policy.clone())?;
    prover3.initialize([0u8; 32])?;

    // Genesis → Admin
    prover3.add_transition(Transition::new(
        [0u8; 32],
        [1u8; 32],
        OriginClass::Admin,
        1000,
    ))?;
    println!("   Genesis → Admin");

    // Multiple Admin → Admin transitions
    let admin_limit = policy.get_rate_limit(OriginClass::Admin);
    println!("  Admin rate limit: {} per epoch", admin_limit);

    for i in 1..=12 {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::Admin,
            1000 + i as u64,
        );

        match prover3.add_transition(t) {
            Ok(_) => println!("   Admin transition #{}", i),
            Err(ZkOriginError::RateLimitExceeded { current, limit, .. }) => {
                println!(
                    "   Admin transition #{}: Rate limit exceeded ({}/{})",
                    i, current, limit
                );
                break;
            }
            Err(e) => {
                println!("   Admin transition #{}: {}", i, e);
                break;
            }
        }
    }

    // Scenario 4: Performance test
    println!("\n Scenario 4: Performance test");
    let mut prover4 = LineageProver::new(policy)?;
    prover4.initialize([0u8; 32])?;

    let num_transitions = 100;
    let start = Instant::now();

    for i in 0..num_transitions {
        let t = Transition::new(
            [i as u8; 32],
            [(i + 1) as u8; 32],
            OriginClass::User,
            (i as u64 + 1) * 1000,
        );
        prover4.add_transition(t)?;
    }

    let transition_time = start.elapsed();
    println!(
        "  Added {} transitions in {:?}",
        num_transitions, transition_time
    );
    println!(
        "  Average: {:?} per transition",
        transition_time / num_transitions
    );

    let start = Instant::now();
    let proof4 = prover4.finalize()?;
    let proof_time = start.elapsed();

    println!("  Generated proof in {:?}", proof_time);
    println!("  Proof size: {} bytes", proof4.proof_size());
    println!("  Depth: {}", proof4.num_steps);

    Ok(())
}
