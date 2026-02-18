//! Lineage proof types

use super::{LineageCommitment, CounterCommitment};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A complete lineage proof that can be verified.
///
/// This is the final output of the prover, containing:
/// - The compressed SNARK proof
/// - Public outputs (lineage commitment, counters)
/// - Metadata for verification
#[derive(Clone, Serialize, Deserialize)]
pub struct LineageProof {
    /// The compressed proof bytes
    pub proof_bytes: Vec<u8>,
    
    /// Final lineage commitment
    pub final_lineage: LineageCommitment,
    
    /// Final counter commitment
    pub final_counters: CounterCommitment,
    
    /// Genesis commitment (for verification)
    pub genesis_commitment: LineageCommitment,
    
    /// Number of steps in the lineage
    pub num_steps: u64,
    
    /// Policy hash used
    pub policy_hash: [u8; 32],
    
    /// Proof metadata
    pub metadata: ProofMetadata,
}

impl LineageProof {
    /// Create a new lineage proof
    pub fn new(
        proof_bytes: Vec<u8>,
        final_lineage: LineageCommitment,
        final_counters: CounterCommitment,
        genesis_commitment: LineageCommitment,
        num_steps: u64,
        policy_hash: [u8; 32],
    ) -> Self {
        Self {
            proof_bytes,
            final_lineage,
            final_counters,
            genesis_commitment,
            num_steps,
            policy_hash,
            metadata: ProofMetadata::default(),
        }
    }

    /// Add metadata to the proof
    pub fn with_metadata(mut self, metadata: ProofMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get the size of the proof in bytes
    pub fn proof_size(&self) -> usize {
        self.proof_bytes.len()
    }

    /// Serialize the proof to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Verify the proof (placeholder - actual verification in verifier module)
    pub fn verify(&self) -> crate::Result<bool> {
        // This is a placeholder. Actual verification happens in verifier::verify
        // For now, we do basic sanity checks
        
        if self.proof_bytes.is_empty() {
            return Err(crate::ZkOriginError::InvalidProof("Empty proof".into()));
        }
        
        if self.num_steps == 0 {
            return Err(crate::ZkOriginError::InvalidProof("Zero steps".into()));
        }
        
        if self.final_lineage.depth != self.num_steps {
            return Err(crate::ZkOriginError::InvalidProof("Depth mismatch".into()));
        }
        
        // In a real implementation, this would verify the SNARK
        Ok(true)
    }

    /// Get a summary of the proof for logging
    pub fn summary(&self) -> ProofSummary {
        ProofSummary {
            lineage_hash: self.final_lineage.to_hex(),
            depth: self.num_steps,
            proof_size: self.proof_size(),
            genesis_hash: self.genesis_commitment.to_hex(),
        }
    }
}

impl fmt::Debug for LineageProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineageProof")
            .field("final_lineage", &self.final_lineage)
            .field("num_steps", &self.num_steps)
            .field("proof_size", &self.proof_size())
            .field("genesis", &self.genesis_commitment)
            .finish()
    }
}

impl fmt::Display for LineageProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LineageProof(depth={}, size={}B, lineage={})",
            self.num_steps,
            self.proof_size(),
            self.final_lineage
        )
    }
}

/// Metadata about the proof generation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// When the proof was generated (Unix timestamp)
    pub generated_at: u64,
    
    /// Time taken to generate in milliseconds
    pub proving_time_ms: u64,
    
    /// Prover version
    pub prover_version: String,
    
    /// Curve used
    pub curve: String,
    
    /// Additional notes
    pub notes: Option<String>,
}

impl ProofMetadata {
    /// Create new metadata with current timestamp
    pub fn new() -> Self {
        Self {
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            prover_version: env!("CARGO_PKG_VERSION").to_string(),
            curve: "Pallas/Vesta".to_string(),
            ..Default::default()
        }
    }

    /// Set proving time
    pub fn with_proving_time(mut self, ms: u64) -> Self {
        self.proving_time_ms = ms;
        self
    }

    /// Set notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Summary of a proof for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSummary {
    /// Hex-encoded lineage hash
    pub lineage_hash: String,
    
    /// Lineage depth
    pub depth: u64,
    
    /// Proof size in bytes
    pub proof_size: usize,
    
    /// Genesis hash
    pub genesis_hash: String,
}

impl fmt::Display for ProofSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Lineage Proof Summary:")?;
        writeln!(f, "  Lineage:  {}...", &self.lineage_hash[..16])?;
        writeln!(f, "  Depth:    {}", self.depth)?;
        writeln!(f, "  Size:     {} bytes", self.proof_size)?;
        writeln!(f, "  Genesis:  {}...", &self.genesis_hash[..16])
    }
}

/// Batch of proofs
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProofBatch {
    /// The proofs
    pub proofs: Vec<LineageProof>,
    
    /// Batch metadata
    pub batch_id: Option<String>,
}

impl ProofBatch {
    /// Create a new batch
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a proof
    pub fn push(&mut self, proof: LineageProof) {
        self.proofs.push(proof);
    }

    /// Get total size
    pub fn total_size(&self) -> usize {
        self.proofs.iter().map(|p| p.proof_size()).sum()
    }

    /// Number of proofs
    pub fn len(&self) -> usize {
        self.proofs.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.proofs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_proof() -> LineageProof {
        LineageProof::new(
            vec![1, 2, 3, 4, 5], // dummy proof bytes
            LineageCommitment::new([1u8; 32], 10),
            CounterCommitment::new([2u8; 32], 0),
            LineageCommitment::genesis([0u8; 32]),
            10,
            [3u8; 32],
        )
    }

    #[test]
    fn test_proof_creation() {
        let proof = create_test_proof();
        
        assert_eq!(proof.num_steps, 10);
        assert_eq!(proof.proof_size(), 5);
        assert_eq!(proof.final_lineage.depth, 10);
    }

    #[test]
    fn test_proof_serialization_bincode() {
        let proof = create_test_proof();
        
        let bytes = proof.to_bytes().unwrap();
        let recovered = LineageProof::from_bytes(&bytes).unwrap();
        
        assert_eq!(proof.num_steps, recovered.num_steps);
        assert_eq!(proof.final_lineage.value, recovered.final_lineage.value);
    }

    #[test]
    fn test_proof_serialization_json() {
        let proof = create_test_proof();
        
        let json = proof.to_json().unwrap();
        let recovered = LineageProof::from_json(&json).unwrap();
        
        assert_eq!(proof.num_steps, recovered.num_steps);
    }

    #[test]
    fn test_proof_summary() {
        let proof = create_test_proof();
        let summary = proof.summary();
        
        assert_eq!(summary.depth, 10);
        assert_eq!(summary.proof_size, 5);
    }

    #[test]
    fn test_metadata() {
        let metadata = ProofMetadata::new()
            .with_proving_time(1234)
            .with_notes("Test proof");
        
        assert_eq!(metadata.proving_time_ms, 1234);
        assert_eq!(metadata.notes, Some("Test proof".to_string()));
    }

    #[test]
    fn test_proof_verify_basic() {
        let proof = create_test_proof();
        
        // Should pass basic validation
        assert!(proof.verify().is_ok());
    }

    #[test]
    fn test_proof_verify_empty() {
        let mut proof = create_test_proof();
        proof.proof_bytes = vec![];
        
        assert!(proof.verify().is_err());
    }

    #[test]
    fn test_proof_batch() {
        let mut batch = ProofBatch::new();
        
        batch.push(create_test_proof());
        batch.push(create_test_proof());
        
        assert_eq!(batch.len(), 2);
        assert_eq!(batch.total_size(), 10);
    }
}