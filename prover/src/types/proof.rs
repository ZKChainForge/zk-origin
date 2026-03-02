//! Lineage proof types

use crate::types::lineage::{LineageCommitment, CounterCommitment};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A complete lineage proof that can be verified.
#[derive(Clone, Serialize, Deserialize)]
pub struct LineageProof {
    /// The proof bytes (compressed SNARK or commitment hash)
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
    
    /// Verifier key bytes (for Nova proofs)
    #[serde(default)]
    pub verifier_key: Option<Vec<u8>>,
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
            verifier_key: None,
        }
    }

    /// Add metadata to the proof
    pub fn with_metadata(mut self, metadata: ProofMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add verifier key
    pub fn with_verifier_key(mut self, vk: Vec<u8>) -> Self {
        self.verifier_key = Some(vk);
        self
    }

    /// Get the size of the proof in bytes
    pub fn proof_size(&self) -> usize {
        self.proof_bytes.len()
    }

    /// Check if this is a real ZK proof (vs commitment)
    pub fn is_real_zk(&self) -> bool {
        // Real Nova proofs are at least 1KB
        // Commitment "proofs" are 32 bytes
        self.proof_bytes.len() > 1000
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

    /// Basic verification (structure only)
    /// 
    /// For real cryptographic verification, use LineageVerifier
    pub fn verify(&self) -> crate::Result<bool> {
        // Basic structural checks
        if self.proof_bytes.is_empty() {
            return Err(crate::ZkOriginError::InvalidProof("Empty proof".into()));
        }
        if self.num_steps == 0 {
            return Err(crate::ZkOriginError::InvalidProof("Zero steps".into()));
        }
        if self.final_lineage.depth != self.num_steps {
            return Err(crate::ZkOriginError::InvalidProof("Depth mismatch".into()));
        }
        
        // For real ZK proofs, we'd need the public params to verify
        // This just does structural validation
        if self.is_real_zk() && self.verifier_key.is_none() {
            return Err(crate::ZkOriginError::InvalidProof(
                "Real ZK proof requires verifier key".into()
            ));
        }
        
        Ok(true)
    }

    /// Get a summary of the proof
    pub fn summary(&self) -> ProofSummary {
        ProofSummary {
            lineage_hash: self.final_lineage.to_hex(),
            depth: self.num_steps,
            proof_size: self.proof_size(),
            genesis_hash: self.genesis_commitment.to_hex(),
            is_real_zk: self.is_real_zk(),
        }
    }
}

impl fmt::Debug for LineageProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineageProof")
            .field("final_lineage", &self.final_lineage)
            .field("num_steps", &self.num_steps)
            .field("proof_size", &self.proof_size())
            .field("is_real_zk", &self.is_real_zk())
            .field("genesis", &self.genesis_commitment)
            .finish()
    }
}

impl fmt::Display for LineageProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LineageProof(depth={}, size={}B, zk={}, lineage={})",
            self.num_steps,
            self.proof_size(),
            self.is_real_zk(),
            self.final_lineage
        )
    }
}

/// Metadata about the proof generation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// UNIX timestamp when generated
    pub generated_at: u64,
    
    /// Proving time in milliseconds
    pub proving_time_ms: u64,
    
    /// Prover version
    pub prover_version: String,
    
    /// Curve used
    pub curve: String,
    
    /// Optional notes
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

/// Summary of a proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSummary {
    /// Lineage hash
    pub lineage_hash: String,
    /// Depth
    pub depth: u64,
    /// Proof size
    pub proof_size: usize,
    /// Genesis hash
    pub genesis_hash: String,
    /// Whether real ZK
    pub is_real_zk: bool,
}

impl fmt::Display for ProofSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Lineage Proof Summary:")?;
        writeln!(f, "  Lineage:  {}...", &self.lineage_hash[..16.min(self.lineage_hash.len())])?;
        writeln!(f, "  Depth:    {}", self.depth)?;
        writeln!(f, "  Size:     {} bytes", self.proof_size)?;
        writeln!(f, "  Real ZK:  {}", self.is_real_zk)?;
        writeln!(f, "  Genesis:  {}...", &self.genesis_hash[..16.min(self.genesis_hash.len())])
    }
}

/// Batch of proofs
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProofBatch {
    /// Proofs in batch
    pub proofs: Vec<LineageProof>,
    /// Batch ID
    pub batch_id: Option<String>,
}