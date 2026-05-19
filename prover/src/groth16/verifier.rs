
use super::Proof;
use crate::error::{ProverError, Result};
use std::process::Command;
use std::fs;

/// Groth16 verifier
pub struct Groth16Verifier {
    vk_path: String,
}

impl Groth16Verifier {
    /// Create new verifier
    pub fn new(vk_path: String) -> Self {
        Groth16Verifier { vk_path }
    }
    
    /// Verify proof
    pub fn verify(
        &self,
        proof: &Proof,
        public_signals: &[String],
    ) -> Result<bool> {
        // Save proof and public signals to temporary files
        let proof_json = serde_json::to_string_pretty(proof)
            .map_err(|e| ProverError::SerializationError(e.to_string()))?;
        
        let signals_json = serde_json::to_string_pretty(public_signals)
            .map_err(|e| ProverError::SerializationError(e.to_string()))?;
        
        fs::write("proof.json", proof_json)?;
        fs::write("public.json", signals_json)?;
        
        // Call snarkjs to verify
        let output = Command::new("snarkjs")
            .arg("groth16")
            .arg("verify")
            .arg(&self.vk_path)
            .arg("public.json")
            .arg("proof.json")
            .output()
            .map_err(|e| ProverError::proof_generation_failed(format!("snarkjs failed: {}", e)))?;
        
        let result = output.status.success();
        
        // Clean up
        let _ = fs::remove_file("proof.json");
        let _ = fs::remove_file("public.json");
        
        Ok(result)
    }
}