use std::fmt;

/// Prover-specific errors with detailed context
#[derive(Debug, thiserror::Error)]
pub enum ProverError {
    #[error("Invalid witness: {context}")]
    InvalidWitness { context: String },

    #[error("Invalid state: {context}")]
    InvalidState { context: String },

    #[error("Nonce error: {context}")]
    InvalidNonce { context: String },

    #[error("Timestamp error: {context}")]
    InvalidTimestamp { context: String },

    #[error("Proof generation failed: {context}")]
    ProofGenerationFailed { context: String },

    #[error("Proof verification failed: {context}")]
    ProofVerificationFailed { context: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Rate limit exceeded: {context}")]
    RateLimitExceeded { context: String },

    #[error("Authorization failed: {context}")]
    AuthorizationFailed { context: String },

    #[error("Policy violation: {context}")]
    PolicyViolation { context: String },

    #[error("Batch operation failed: {context}")]
    BatchOperationFailed { context: String },

    #[error("Circuit error: {context}")]
    CircuitError { context: String },

    #[error("Cryptographic error: {context}")]
    CryptoError { context: String },

    #[error("Generic error: {0}")]
    Other(String),
}

impl ProverError {
    pub fn invalid_witness(context: impl Into<String>) -> Self {
        ProverError::InvalidWitness {
            context: context.into(),
        }
    }

    pub fn invalid_state(context: impl Into<String>) -> Self {
        ProverError::InvalidState {
            context: context.into(),
        }
    }

    pub fn invalid_nonce(context: impl Into<String>) -> Self {
        ProverError::InvalidNonce {
            context: context.into(),
        }
    }

    pub fn invalid_timestamp(context: impl Into<String>) -> Self {
        ProverError::InvalidTimestamp {
            context: context.into(),
        }
    }

    pub fn proof_generation_failed(context: impl Into<String>) -> Self {
        ProverError::ProofGenerationFailed {
            context: context.into(),
        }
    }

    pub fn rate_limit_exceeded(context: impl Into<String>) -> Self {
        ProverError::RateLimitExceeded {
            context: context.into(),
        }
    }

    pub fn authorization_failed(context: impl Into<String>) -> Self {
        ProverError::AuthorizationFailed {
            context: context.into(),
        }
    }

    pub fn policy_violation(context: impl Into<String>) -> Self {
        ProverError::PolicyViolation {
            context: context.into(),
        }
    }

    pub fn batch_operation_failed(context: impl Into<String>) -> Self {
        ProverError::BatchOperationFailed {
            context: context.into(),
        }
    }
}

/// Result type
pub type Result<T> = std::result::Result<T, ProverError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ProverError::invalid_witness("test");
        assert!(err.to_string().contains("Invalid witness"));
    }

    #[test]
    fn test_error_display() {
        let err = ProverError::invalid_nonce("nonce not increasing");
        assert!(!err.to_string().is_empty());
    }
}
