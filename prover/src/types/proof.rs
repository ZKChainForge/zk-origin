//! Lineage proof types

use crate::types::lineage::{LineageCommitment, CounterCommitment};
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

    /// Verify the proof (placeholder)
    pub fn verify(&self) -> crate::Result<bool> {
        if self.proof_bytes.is_empty() {
            return Err(crate::ZkOriginError::InvalidProof("Empty proof".into()));
        }
        if self.num_steps == 0 {
            return Err(crate::ZkOriginError::InvalidProof("Zero steps".into()));
        }
        if self.final_lineage.depth != self.num_steps {
            return Err(crate::ZkOriginError::InvalidProof("Depth mismatch".into()));
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
/// Metadata about the proof generation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// UNIX timestamp (seconds) when the proof was generated
    pub generated_at: u64,

    /// Time taken to generate the proof (milliseconds)
    pub proving_time_ms: u64,

    /// Version of the prover software
    pub prover_version: String,

    /// Elliptic curve(s) used for proving
    pub curve: String,

    /// Optional human-readable notes
    pub notes: Option<String>,
}


impl ProofMetadata {
    /// Create a new `ProofMetadata` instance with default values
    /// and the current system timestamp.
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

    /// Set the proving time in milliseconds.
    pub fn with_proving_time(mut self, ms: u64) -> Self {
        self.proving_time_ms = ms;
        self
    }

    /// Attach human-readable notes to the metadata.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}


/// Summary of a proof for display
/// Summary of a proof for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSummary {
    /// Hex-encoded final lineage commitment
    pub lineage_hash: String,

    /// Number of transitions in the lineage
    pub depth: u64,

    /// Size of the proof in bytes
    pub proof_size: usize,

    /// Hex-encoded genesis commitment
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
/// Batch of lineage proofs
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProofBatch {
    /// Collection of lineage proofs
    pub proofs: Vec<LineageProof>,

    /// Optional identifier for the batch
    pub batch_id: Option<String>,
}
