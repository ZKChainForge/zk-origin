//! Groth16 verifier

use super::Proof;
use crate::Result;
use std::process::Command;

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
            .map_err(|e| crate::Error::SerializationError(e.to_string()))?;
        
        let signals_json = serde_json::to_string_pretty(public_signals)
            .map_err(|e| crate::Error::SerializationError(e.to_string()))?;
        
        std::fs::write("proof.json", proof_json)?;
        std::fs::write("public.json", signals_json)?;
        
        // Call snarkjs to verify
        let output = Command::new("snarkjs")
            .arg("groth16")
            .arg("verify")
            .arg(&self.vk_path)
            .arg("public.json")
            .arg("proof.json")
            .output()
            .map_err(|e| crate::Error::ProofGenerationFailed(format!("snarkjs failed: {}", e)))?;
        
        let result = output.status.success();
        
        // Clean up
        let _ = std::fs::remove_file("proof.json");
        let _ = std::fs::remove_file("public.json");
        
        Ok(result)
    }
}