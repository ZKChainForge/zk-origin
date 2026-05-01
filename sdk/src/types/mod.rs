//! Type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transition data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    /// Previous state hash
    pub prev_state_hash: [u8; 32],
    
    /// New state hash
    pub new_state_hash: [u8; 32],
    
    /// Origin class
    pub origin_class: u8,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Nonce
    pub nonce: u64,
}

/// Witness for circuit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Witness {
    /// Public inputs
    pub public: PublicInputs,
    
    /// Private inputs
    pub private: PrivateInputs,
}

/// Public inputs (visible in proof)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicInputs {
    /// New lineage commitment
    pub new_lineage_commitment: String,
    
    /// New counter commitment
    pub new_counter_commitment: String,
    
    /// Lineage valid
    pub lineage_valid: u32,
    
    /// Previous state hash
    pub prev_state_hash: String,
    
    /// New state hash
    pub new_state_hash: String,
    
    /// Epoch ID
    pub epoch_id: u32,
    
    /// Previous origin class
    pub prev_origin_class: u8,
    
    /// New origin class
    pub new_origin_class: u8,
    
    /// Previous lineage commitment
    pub prev_lineage_commitment: String,
    
    /// Previous counter commitment
    pub prev_counter_commitment: String,
    
    /// Policy root
    pub policy_root: String,
    
    /// Expected genesis hash
    pub expected_genesis_hash: String,
}

/// Private inputs (hidden in proof)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateInputs {
    /// Previous epoch ID
    pub prev_epoch_id: u32,
    
    /// Previous depth
    pub prev_depth: u32,
    
    /// Nonce
    pub nonce: u64,
    
    /// Previous nonce
    pub prev_nonce: u64,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Previous timestamp
    pub prev_timestamp: u64,
    
    /// Policy Merkle proof
    pub policy_proof: Vec<String>,
    
    /// Policy indices
    pub policy_indices: Vec<u8>,
    
    /// Previous counters
    pub prev_counters: Vec<u32>,
    
    /// Rate limits
    pub rate_limits: Vec<u32>,
    
    /// Public key X
    pub public_key_x: Option<String>,
    
    /// Public key Y
    pub public_key_y: Option<String>,
    
    /// Signature R
    pub signature_r: Option<String>,
    
    /// Signature S
    pub signature_s: Option<String>,
    
    /// Authorization valid
    pub authorization_valid: u32,
}

/// Groth16 Proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof {
    /// A point
    pub pi_a: [String; 2],
    
    /// B point
    pub pi_b: [[String; 2]; 2],
    
    /// C point
    pub pi_c: [String; 2],
    
    /// Protocol
    pub protocol: String,
}

/// Proof submission
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofSubmission {
    /// Proof
    pub proof: Proof,
    
    /// Public inputs
    pub public_inputs: Vec<String>,
    
    /// Transaction hash
    pub tx_hash: String,
    
    /// Status
    pub status: String,
}

/// State record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateRecord {
    /// State hash
    pub hash: [u8; 32],
    
    /// Lineage commitment
    pub lineage_commitment: [u8; 32],
    
    /// Depth
    pub depth: u64,
    
    /// Origin class
    pub origin_class: u8,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Creator
    pub creator: String,
    
    /// Verified
    pub verified: bool,
}

/// Lineage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lineage {
    /// Genesis hash
    pub genesis: [u8; 32],
    
    /// Depth
    pub depth: u64,
    
    /// States
    pub states: Vec<StateRecord>,
}

/// Contract statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractStats {
    /// Total transitions verified
    pub total_transitions: u64,
    
    /// Max depth reached
    pub max_depth: u64,
    
    /// Genesis initialized
    pub genesis_initialized: bool,
    
    /// Contract paused
    pub paused: bool,
    
    /// Current epoch
    pub current_epoch: u64,
}

/// Lineage verification result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Is valid
    pub valid: bool,
    
    /// Message
    pub message: String,
    
    /// Lineage
    pub lineage: Option<Lineage>,
}