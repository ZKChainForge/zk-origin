//! Error types

use thiserror::Error;

/// Prover error type
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid witness
    #[error("Invalid witness: {0}")]
    InvalidWitness(String),
    
    /// Proof generation failed
    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;