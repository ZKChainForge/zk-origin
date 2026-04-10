use zk_origin::proof::{Transition, LineageCommitment, Witness};
use zk_origin::types::OriginClass;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════╗");
    println!("║   ZK-ORIGIN Witness Generator             ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Genesis state (FIXED hash)
    let genesis_hash =
        "0x625c5b236fc76adb52cfca20ca3928821d56c24bd9719b27415608b9a036ebf4".to_string();

    println!(" Creating Genesis Transition...\n");

    // Create genesis lineage
    let genesis_lineage = LineageCommitment::genesis(genesis_hash.clone())?;

    println!(" Genesis lineage created");
    println!("   Commitment: {}", genesis_lineage.commitment);
    println!("   Depth: {}\n", genesis_lineage.depth);

    // Create first user transition
    let user_state =
        "0x625c5b236fc76adb52cfca20ca3928821d56c24bd9719b27415608b9a036ebf5".to_string();

    let transition = Transition::new(
        genesis_hash.clone(),
        user_state.clone(),
        OriginClass::Genesis,
        OriginClass::User,
        0,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        1,
        0,
    )?;

    println!(" Transition created");
    println!("   From: {}", transition.prev_origin_class);
    println!("   To: {}", transition.new_origin_class);
    println!("   Hash: {}\n", transition.hash());

    // Update lineage
    let updated_lineage = genesis_lineage.update(&transition.hash())?;

    println!(" Lineage updated");
    println!("   New commitment: {}", updated_lineage.commitment);
    println!("   New depth: {}\n", updated_lineage.depth);

    // Create witness with ALL required fields
    let witness = Witness::from_transition(
        &transition,
        &genesis_lineage,
        &updated_lineage,
        vec![0, 0, 0, 0, 0, 0, 0],              // prevCounters (7 values)
        vec![1, 4294967295, 10, 100, 5, 1000, 1], // rateLimits (7 values)
        "0x000000000000000000000000000000000000000000000000d8e770f2f5a1ff14"
            .to_string(),
    )?;



    // Print witness structure for debugging
    println!("\n Witness Structure:");
    println!("   prevStateHash: {}", witness.prev_state_hash);
    println!("   newStateHash: {}", witness.new_state_hash);
    println!("   prevOriginClass: {}", witness.prev_origin_class);
    println!("   newOriginClass: {}", witness.new_origin_class);
    println!("   authorizationValid: {}", witness.authorization_valid);
    println!("   prevCounters length: {}", witness.prev_counters.len());
    println!("   rateLimits length: {}", witness.rate_limits.len());
    println!("   policyProof length: {}", witness.policy_proof.len());
    println!("   policyIndices length: {}", witness.policy_indices.len());

    // Save witness
    witness.save_to_file("circuits/test/inputs/first_transition_witness.json")?;


    // Create circuit input JSON
    create_circuit_input(&witness)?;

    Ok(())
}

fn create_circuit_input(witness: &Witness) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;

    // Create properly formatted circuit input
    let input = json!({
        "prevStateHash": witness.prev_state_hash,
        "newStateHash": witness.new_state_hash,
        "epochId": witness.epoch_id,
        "prevOriginClass": witness.prev_origin_class,
        "newOriginClass": witness.new_origin_class,
        "prevLineageCommitment": witness.prev_lineage_commitment,
        "prevCounterCommitment": witness.prev_counter_commitment,
        "policyRoot": witness.policy_root,
        "expectedGenesisHash": witness.expected_genesis_hash,
        
        // Private inputs
        "prevEpochId": witness.prev_epoch_id,
        "prevDepth": witness.prev_depth,
        "nonce": witness.nonce,
        "prevNonce": witness.prev_nonce,
        "timestamp": witness.timestamp,
        "prevTimestamp": witness.prev_timestamp,
        "policyProof": witness.policy_proof,
        "policyIndices": witness.policy_indices,
        "prevCounters": witness.prev_counters,
        "rateLimits": witness.rate_limits,
        "authorizationValid": witness.authorization_valid,
    });

    let input_path = "circuits/test/inputs/first_transition_input.json";
    fs::write(input_path, serde_json::to_string_pretty(&input)?)?;

    println!(" Circuit input saved to: {}", input_path);

    Ok(())
}