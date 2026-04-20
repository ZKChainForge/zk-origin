//! Groth16 prover

use crate::{error::{Error, Result}, witness::generator::TransitionWitness};
use super::Proof;
use std::process::Command;
use tokio::fs;

/// Groth16 prover
pub struct Groth16Prover {
    circuit_path: String,
    zkey_path: String,
}

impl Groth16Prover {
    /// Create new prover
    pub fn new(circuit_path: String, zkey_path: String) -> Self {
        Groth16Prover {
            circuit_path,
            zkey_path,
        }
    }
    
    /// Generate proof
    pub async fn prove(&self, witness: &TransitionWitness) -> Result<Proof> {
        // Save witness to temporary file
        let witness_json = serde_json::to_string_pretty(witness)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        
        fs::write("witness.json", witness_json).await
            .map_err(|e| Error::IoError(e))?;
        
        // Call snarkjs to generate proof
        let output = Command::new("snarkjs")
            .arg("groth16")
            .arg("prove")
            .arg(&self.zkey_path)
            .arg("witness.json")
            .arg("proof.json")
            .arg("public.json")
            .output()
            .map_err(|e| Error::ProofGenerationFailed(format!("snarkjs failed: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ProofGenerationFailed(format!("snarkjs error: {}", stderr)));
        }
        
        // Read proof file
        let proof_json = fs::read_to_string("proof.json").await
            .map_err(|e| Error::IoError(e))?;
        
        let proof: Proof = serde_json::from_str(&proof_json)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        
        // Clean up
        let _ = fs::remove_file("witness.json").await;
        let _ = fs::remove_file("proof.json").await;
        let _ = fs::remove_file("public.json").await;
        
        Ok(proof)
    }
}