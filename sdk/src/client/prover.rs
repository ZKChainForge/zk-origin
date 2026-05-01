//! Prover client for witness and proof generation

use crate::config::Config;
use crate::types::*;
use crate::error::{Error, Result};
use crate::utils::hash::Keccak256;
use std::process::Command;
use tokio::fs;

/// Prover client
pub struct ProverClient {
    config: Config,
    initialized: bool,
}

impl ProverClient {
    /// Create new prover client
    pub fn new(config: Config) -> Self {
        ProverClient {
            config,
            initialized: false,
        }
    }
    
    /// Initialize
    pub async fn initialize(&mut self) -> Result<()> {
        // Verify prover binary exists
        let output = Command::new(&self.config.prover_binary)
            .arg("--version")
            .output()
            .map_err(|e| Error::ProofError(format!("Failed to verify prover: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::ProofError("Prover binary not found".to_string()));
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Generate witness
    pub async fn generate_witness(
        &self,
        prev_state: &[u8],
        new_state: &[u8],
        origin_class: u8,
    ) -> Result<Witness> {
        if !self.initialized {
            return Err(Error::WitnessError("Prover not initialized".to_string()));
        }
        
        // Hash states
        let prev_hash = Keccak256::hash(prev_state);
        let new_hash = Keccak256::hash(new_state);
        
        // Create witness structure
        let witness = Witness {
            public: PublicInputs {
                new_lineage_commitment: "0".to_string(),
                new_counter_commitment: "0".to_string(),
                lineage_valid: 1,
                prev_state_hash: format!("{}", hex::encode(&prev_hash[..])),
                new_state_hash: format!("{}", hex::encode(&new_hash[..])),
                epoch_id: 0,
                prev_origin_class: 0,
                new_origin_class: origin_class,
                prev_lineage_commitment: "0".to_string(),
                prev_counter_commitment: "0".to_string(),
                policy_root: self.config.policy_root.clone(),
                expected_genesis_hash: self.config.genesis_hash.clone(),
            },
            private: PrivateInputs {
                prev_epoch_id: 0,
                prev_depth: 0,
                nonce: 1,
                prev_nonce: 0,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                prev_timestamp: 0,
                policy_proof: vec![],
                policy_indices: vec![],
                prev_counters: vec![0, 0, 0, 0, 0, 0, 0],
                rate_limits: vec![1, u32::MAX, 10, 100, 5, 1000, 1],
                public_key_x: None,
                public_key_y: None,
                signature_r: None,
                signature_s: None,
                authorization_valid: 1,
            },
        };
        
        Ok(witness)
    }
    
    /// Generate proof
    pub async fn generate_proof(&self, witness: &Witness) -> Result<Proof> {
        if !self.initialized {
            return Err(Error::ProofError("Prover not initialized".to_string()));
        }
        
        // Save witness to file
        let witness_json = serde_json::to_string(witness)
            .map_err(|e| Error::ProofError(format!("Failed to serialize witness: {}", e)))?;
        
        fs::write("witness.json", witness_json).await
            .map_err(|e| Error::ProofError(format!("Failed to write witness: {}", e)))?;
        
        // Call snarkjs to generate proof
        let output = Command::new("snarkjs")
            .arg("groth16")
            .arg("prove")
            .arg(format!("{}/main_final.zkey", self.config.circuit_path))
            .arg("witness.json")
            .arg("proof.json")
            .arg("public.json")
            .output()
            .map_err(|e| Error::ProofError(format!("snarkjs failed: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ProofError(format!("snarkjs error: {}", stderr)));
        }
        
        // Read proof file
        let proof_json = fs::read_to_string("proof.json").await
            .map_err(|e| Error::ProofError(format!("Failed to read proof: {}", e)))?;
        
        let proof: Proof = serde_json::from_str(&proof_json)
            .map_err(|e| Error::ProofError(format!("Failed to parse proof: {}", e)))?;
        
        Ok(proof)
    }
}