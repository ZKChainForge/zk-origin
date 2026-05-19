
use crate::error::{ProverError, Result};
use crate::witness::generator::TransitionWitness;
use super::Proof;
use std::process::Command;
use std::fs;

/// Groth16 prover
pub struct Groth16Prover {
    /// Circuit file path
    #[allow(dead_code)]
    circuit_path: String,
    /// Zero-knowledge key path
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
    pub fn prove(&self, witness: &TransitionWitness) -> Result<Proof> {
        // Save witness to temporary file
        let witness_json = serde_json::to_string_pretty(witness)
            .map_err(|e| ProverError::SerializationError(e.to_string()))?;
        
        fs::write("witness.json", witness_json)?;
        
        // Call snarkjs to generate proof
        let output = Command::new("snarkjs")
            .arg("groth16")
            .arg("prove")
            .arg(&self.zkey_path)
            .arg("witness.json")
            .arg("proof.json")
            .arg("public.json")
            .output()
            .map_err(|e| ProverError::proof_generation_failed(format!("snarkjs failed: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProverError::proof_generation_failed(format!("snarkjs error: {}", stderr)));
        }
        
        // Read proof file
        let proof_json = fs::read_to_string("proof.json")?;
        
        let proof: Proof = serde_json::from_str(&proof_json)
            .map_err(|e| ProverError::SerializationError(e.to_string()))?;
        
        // Clean up
        let _ = fs::remove_file("witness.json");
        let _ = fs::remove_file("proof.json");
        let _ = fs::remove_file("public.json");
        
        Ok(proof)
    }
}