//! Serialization utilities

use crate::types::*;
use crate::error::{Error, Result};

/// Witness serializer
pub struct WitnessSerializer;

impl WitnessSerializer {
    /// Serialize to JSON
    pub fn to_json(witness: &Witness) -> Result<String> {
        serde_json::to_string_pretty(witness)
            .map_err(|e| Error::SerializationError(e.to_string()))
    }
    
    /// Serialize to JSON with formatting for Circom
    pub fn to_circom_format(witness: &Witness) -> Result<String> {
        let json = serde_json::json!({
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
            "authorizationValid": witness.private.authorization_valid,
        });
        
        serde_json::to_string_pretty(&json)
            .map_err(|e| Error::SerializationError(e.to_string()))
    }
}

/// Proof formatter
pub struct ProofFormatter;

impl ProofFormatter {
    /// Format proof for contract
    pub fn to_solidity_call(_proof: &Proof, _public_inputs: &[String]) -> Result<String> {
        // TODO: Format as contract call
        Ok(String::new())
    }
}