//! Error types for the prover

use thiserror::Error;

/// Result type for prover operations
pub type Result<T> = std::result::Result<T, ProverError>;

/// Prover error types
#[derive(Debug, Error)]
pub enum ProverError {
    #[error("Invalid nonce: {0}")]
    InvalidNonce(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Invalid witness: {0}")]
    InvalidWitness(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Invalid hash: {0}")]
    InvalidHash(String),

    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl ProverError {
    pub fn invalid_nonce(msg: impl Into<String>) -> Self {
        ProverError::InvalidNonce(msg.into())
    }

    pub fn invalid_timestamp(msg: impl Into<String>) -> Self {
        ProverError::InvalidTimestamp(msg.into())
    }

    pub fn invalid_state(msg: impl Into<String>) -> Self {
        ProverError::InvalidState(msg.into())
    }

    pub fn invalid_witness(msg: impl Into<String>) -> Self {
        ProverError::InvalidWitness(msg.into())
    }

    pub fn rate_limit_exceeded(msg: impl Into<String>) -> Self {
        ProverError::RateLimitExceeded(msg.into())
    }

    pub fn authorization_failed(msg: impl Into<String>) -> Self {
        ProverError::AuthorizationFailed(msg.into())
    }

    pub fn invalid_hash(msg: impl Into<String>) -> Self {
        ProverError::InvalidHash(msg.into())
    }

    pub fn proof_generation_failed(msg: impl Into<String>) -> Self {
        ProverError::ProofGenerationFailed(msg.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        ProverError::InternalError(msg.into())
    }
}