//! Witness generation binary for ZK-ORIGIN

use log::{error, info};
use serde_json::json;
use std::io::{self, Read};
use zk_origin_prover::hash::Hash;
use zk_origin_prover::WitnessGenerator;

/// Input structure for witness generation
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WitnessInput {
    prev_state_hash: String,
    new_state_hash: String,
    prev_origin_class: u8,
    new_origin_class: u8,
    prev_lineage_commitment: String,
    prev_counter_commitment: String,
    prev_counters: Vec<u32>,
    prev_depth: u32,
    epoch_id: u32,
    nonce: u64,
    prev_nonce: u64,
    timestamp: u64,
    prev_timestamp: u64,
    policy_merkle_proof: Vec<String>,
    policy_indices: Vec<u8>,
    policy_root: String,
    genesis_hash: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Witness generator started");

    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Parse JSON input
    let witness_input: WitnessInput = serde_json::from_str(&input).map_err(|e| {
        error!("Failed to parse JSON input: {}", e);
        e
    })?;

    info!("Parsed witness input successfully");

    // Parse hashes from hex strings
    let prev_state_hash = Hash::from_hex(&witness_input.prev_state_hash).map_err(|e| {
        error!("Invalid prev_state_hash: {}", e);
        e
    })?;

    let new_state_hash = Hash::from_hex(&witness_input.new_state_hash).map_err(|e| {
        error!("Invalid new_state_hash: {}", e);
        e
    })?;

    let prev_lineage_commitment =
        Hash::from_hex(&witness_input.prev_lineage_commitment).map_err(|e| {
            error!("Invalid prev_lineage_commitment: {}", e);
            e
        })?;

    let prev_counter_commitment =
        Hash::from_hex(&witness_input.prev_counter_commitment).map_err(|e| {
            error!("Invalid prev_counter_commitment: {}", e);
            e
        })?;

    let policy_root = Hash::from_hex(&witness_input.policy_root).map_err(|e| {
        error!("Invalid policy_root: {}", e);
        e
    })?;

    let genesis_hash = Hash::from_hex(&witness_input.genesis_hash).map_err(|e| {
        error!("Invalid genesis_hash: {}", e);
        e
    })?;

    // Parse policy merkle proof
    let policy_merkle_proof: Result<Vec<Hash>, _> = witness_input
        .policy_merkle_proof
        .iter()
        .map(|h| Hash::from_hex(h))
        .collect();

    let policy_merkle_proof = policy_merkle_proof.map_err(|e| {
        error!("Invalid policy merkle proof: {}", e);
        e
    })?;

    info!("All hashes parsed successfully");

    // Validate counter length
    if witness_input.prev_counters.len() != 7 {
        error!(
            "Invalid prev_counters length: expected 7, got {}",
            witness_input.prev_counters.len()
        );
        return Err("Invalid counter count".into());
    }

    // Create witness generator
    let generator = WitnessGenerator::new(policy_root, genesis_hash);
    info!("Witness generator created");

    // Generate witness
    match generator.generate(
        prev_state_hash,
        new_state_hash,
        witness_input.prev_origin_class,
        witness_input.new_origin_class,
        prev_lineage_commitment,
        prev_counter_commitment,
        witness_input.prev_counters,
        witness_input.prev_depth,
        witness_input.epoch_id,
        witness_input.nonce,
        witness_input.prev_nonce,
        witness_input.timestamp,
        witness_input.prev_timestamp,
        policy_merkle_proof,
        witness_input.policy_indices,
    ) {
        Ok(witness) => {
            info!("Witness generated successfully");

            // Convert witness to JSON
            match witness.to_json() {
                Ok(witness_json) => {
                    let output = json!({
                        "status": "success",
                        "witness": witness_json
                    });

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                Err(e) => {
                    error!("Failed to serialize witness: {}", e);
                    let output = json!({
                        "status": "error",
                        "error": format!("Serialization failed: {}", e)
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            error!("Failed to generate witness: {}", e);
            let output = json!({
                "status": "error",
                "error": e.to_string()
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Err(e.into());
        }
    }

    Ok(())
}