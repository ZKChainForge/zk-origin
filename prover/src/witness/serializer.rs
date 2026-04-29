//! Serialize witness to JSON for circuit

use super::generator::TransitionWitness;
use serde_json::json;

/// Witness serializer
pub struct WitnessSerializer;

impl WitnessSerializer {
    /// Serialize witness to JSON (Circom format)
    pub fn to_json(witness: &TransitionWitness) -> serde_json::Value {
        json!({
            // Public inputs (in order)
            "newLineageCommitment": witness.public.new_lineage_commitment,
            "newCounterCommitment": witness.public.new_counter_commitment,
            "lineageValid": witness.public.lineage_valid,
            "prevStateHash": witness.public.prev_state_hash,
            "newStateHash": witness.public.new_state_hash,
            "epochId": witness.public.epoch_id,
            "prevOriginClass": witness.public.prev_origin_class,
            "newOriginClass": witness.public.new_origin_class,
            "prevLineageCommitment": witness.public.prev_lineage_commitment,
            "prevCounterCommitment": witness.public.prev_counter_commitment,
            "policyRoot": witness.public.policy_root,
            "expectedGenesisHash": witness.public.expected_genesis_hash,
            
            // Private inputs
            "prevEpochId": witness.private.prev_epoch_id,
            "prevDepth": witness.private.prev_depth,
            "nonce": witness.private.nonce,
            "prevNonce": witness.private.prev_nonce,
            "timestamp": witness.private.timestamp,
            "prevTimestamp": witness.private.prev_timestamp,
            "policyProof": witness.private.policy_proof,
            "policyIndices": witness.private.policy_indices,
            "prevCounters": witness.private.prev_counters,
            "rateLimits": witness.private.rate_limits,
            "publicKeyX": witness.private.public_key_x.as_ref().map(|s| s.as_str()).unwrap_or("0"),
            "publicKeyY": witness.private.public_key_y.as_ref().map(|s| s.as_str()).unwrap_or("0"),
            "signatureR": witness.private.signature_r.as_ref().map(|s| s.as_str()).unwrap_or("0"),
            "signatureS": witness.private.signature_s.as_ref().map(|s| s.as_str()).unwrap_or("0"),
            "authorizationValid": witness.private.authorization_valid,
        })
    }
    
    /// Serialize as Circom input file format
    pub fn to_circom_input(witness: &TransitionWitness) -> String {
        let json = Self::to_json(witness);
        serde_json::to_string_pretty(&json).expect("Failed to serialize")
    }
}