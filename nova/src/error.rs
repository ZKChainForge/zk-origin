use std::fmt;

/// Nova-specific errors with detailed context
#[derive(Debug, thiserror::Error)]
pub enum NovaError {
    #[error("Invalid state size: expected {expected} bytes, got {actual} bytes")]
    InvalidStateSize { expected: usize, actual: usize },

    #[error("Setup failed: {context}")]
    SetupFailed { context: String },

    #[error("Proof generation failed: {context}")]
    ProveFailed { context: String },

    #[error("Proof compression failed: {context}")]
    CompressionFailed { context: String },

    #[error("Proof verification failed: {context}")]
    VerificationFailed { context: String },

    #[error("No proof has been generated yet")]
    NoProofGenerated,

    #[error("Serialization failed: {0}")]
    SerializationError(#[from] bincode::Error),

    #[error("Invalid proof data: {context}")]
    InvalidProofData { context: String },

    #[error("State mismatch: expected {expected}, got {actual}")]
    StateMismatch { expected: String, actual: String },

    #[error("Circuit hash mismatch: expected {expected}, got {actual}")]
    CircuitHashMismatch { expected: String, actual: String },

    #[error("Overflow error: {context}")]
    Overflow { context: String },

    #[error("Underflow error: {context}")]
    Underflow { context: String },

    #[error("Proof tampering detected")]
    ProofTampering,

    #[error("Generic error: {0}")]
    Other(String),
}

impl NovaError {
    pub fn invalid_state_size(expected: usize, actual: usize) -> Self {
        NovaError::InvalidStateSize { expected, actual }
    }

    pub fn setup_failed(context: impl Into<String>) -> Self {
        NovaError::SetupFailed {
            context: context.into(),
        }
    }

    pub fn prove_failed(context: impl Into<String>) -> Self {
        NovaError::ProveFailed {
            context: context.into(),
        }
    }

    pub fn compression_failed(context: impl Into<String>) -> Self {
        NovaError::CompressionFailed {
            context: context.into(),
        }
    }

    pub fn verification_failed(context: impl Into<String>) -> Self {
        NovaError::VerificationFailed {
            context: context.into(),
        }
    }

    pub fn invalid_proof_data(context: impl Into<String>) -> Self {
        NovaError::InvalidProofData {
            context: context.into(),
        }
    }

    pub fn state_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        NovaError::StateMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    pub fn overflow(context: impl Into<String>) -> Self {
        NovaError::Overflow {
            context: context.into(),
        }
    }
}

/// Result type
pub type Result<T> = std::result::Result<T, NovaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = NovaError::invalid_state_size(48, 64);
        assert!(err.to_string().contains("expected 48"));
    }

    #[test]
    fn test_error_display() {
        let err = NovaError::NoProofGenerated;
        assert!(!err.to_string().is_empty());
    }
}
