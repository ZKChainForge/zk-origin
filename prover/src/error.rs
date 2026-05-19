//! Error types for the prover

use thiserror::Error;

/// Result type for prover operations
pub type Result<T> = std::result::Result<T, ProverError>;

/// Prover error types
#[derive(Debug, Error)]
pub enum ProverError {
    /// Invalid nonce error
    #[error("Invalid nonce: {0}")]
    InvalidNonce(String),

    /// Invalid timestamp error
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    /// Invalid state error
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Invalid witness error
    #[error("Invalid witness: {0}")]
    InvalidWitness(String),

    /// Rate limit exceeded error
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Authorization failed error
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Invalid hash error
    #[error("Invalid hash: {0}")]
    InvalidHash(String),

    /// Proof generation failed error
    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Batch operation failed error
    #[error("Batch operation failed: {0}")]
    BatchOperationFailed(String),

    /// Crypto error
    #[error("Crypto error: {context}")]
    CryptoError {
        /// Error context
        context: String
    },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Implement From for serde_json::Error
impl From<serde_json::Error> for ProverError {
    fn from(err: serde_json::Error) -> Self {
        ProverError::SerializationError(err.to_string())
    }
}

// Implement From for hex::FromHexError
impl From<hex::FromHexError> for ProverError {
    fn from(err: hex::FromHexError) -> Self {
        ProverError::InvalidHash(err.to_string())
    }
}

impl ProverError {
    /// Create invalid nonce error
    pub fn invalid_nonce(msg: impl Into<String>) -> Self {
        ProverError::InvalidNonce(msg.into())
    }

    /// Create invalid timestamp error
    pub fn invalid_timestamp(msg: impl Into<String>) -> Self {
        ProverError::InvalidTimestamp(msg.into())
    }

    /// Create invalid state error
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        ProverError::InvalidState(msg.into())
    }

    /// Create invalid witness error
    pub fn invalid_witness(msg: impl Into<String>) -> Self {
        ProverError::InvalidWitness(msg.into())
    }

    /// Create rate limit exceeded error
    pub fn rate_limit_exceeded(msg: impl Into<String>) -> Self {
        ProverError::RateLimitExceeded(msg.into())
    }

    /// Create authorization failed error
    pub fn authorization_failed(msg: impl Into<String>) -> Self {
        ProverError::AuthorizationFailed(msg.into())
    }

    /// Create invalid hash error
    pub fn invalid_hash(msg: impl Into<String>) -> Self {
        ProverError::InvalidHash(msg.into())
    }

    /// Create proof generation failed error
    pub fn proof_generation_failed(msg: impl Into<String>) -> Self {
        ProverError::ProofGenerationFailed(msg.into())
    }

    /// Create internal error
    pub fn internal_error(msg: impl Into<String>) -> Self {
        ProverError::InternalError(msg.into())
    }

    /// Create batch operation failed error
    pub fn batch_operation_failed(msg: impl Into<String>) -> Self {
        ProverError::BatchOperationFailed(msg.into())
    }

    /// Create crypto error
    pub fn crypto_error(msg: impl Into<String>) -> Self {
        ProverError::CryptoError {
            context: msg.into(),
        }
    }
}