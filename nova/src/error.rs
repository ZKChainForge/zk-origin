//! Error handling for Nova IVC operations

/// Nova-specific errors with detailed context
#[derive(Debug, thiserror::Error)]
pub enum NovaError {
    /// Invalid state size error
    #[error("Invalid state size: expected {expected} bytes, got {actual} bytes")]
    InvalidStateSize {
        /// Expected size
        expected: usize,
        /// Actual size
        actual: usize,
    },

    /// Setup failed error
    #[error("Setup failed: {context}")]
    SetupFailed {
        /// Error context
        context: String,
    },

    /// Proof generation failed error
    #[error("Proof generation failed: {context}")]
    ProveFailed {
        /// Error context
        context: String,
    },

    /// Proof compression failed error
    #[error("Proof compression failed: {context}")]
    CompressionFailed {
        /// Error context
        context: String,
    },

    /// Proof verification failed error
    #[error("Proof verification failed: {context}")]
    VerificationFailed {
        /// Error context
        context: String,
    },

    /// No proof generated yet
    #[error("No proof has been generated yet")]
    NoProofGenerated,

    /// Serialization error
    #[error("Serialization failed: {0}")]
    SerializationError(#[from] bincode::Error),

    /// Invalid proof data error
    #[error("Invalid proof data: {context}")]
    InvalidProofData {
        /// Error context
        context: String,
    },

    /// State mismatch error
    #[error("State mismatch: expected {expected}, got {actual}")]
    StateMismatch {
        /// Expected state
        expected: String,
        /// Actual state
        actual: String,
    },

    /// Circuit hash mismatch error
    #[error("Circuit hash mismatch: expected {expected}, got {actual}")]
    CircuitHashMismatch {
        /// Expected hash
        expected: String,
        /// Actual hash
        actual: String,
    },

    /// Overflow error
    #[error("Overflow error: {context}")]
    Overflow {
        /// Error context
        context: String,
    },

    /// Underflow error
    #[error("Underflow error: {context}")]
    Underflow {
        /// Error context
        context: String,
    },

    /// Proof tampering detected
    #[error("Proof tampering detected")]
    ProofTampering,

    /// Generic error
    #[error("Generic error: {0}")]
    Other(String),
}

impl NovaError {
    /// Create an invalid state size error
    pub fn invalid_state_size(expected: usize, actual: usize) -> Self {
        NovaError::InvalidStateSize { expected, actual }
    }

    /// Create a setup failed error
    pub fn setup_failed(context: impl Into<String>) -> Self {
        NovaError::SetupFailed {
            context: context.into(),
        }
    }

    /// Create a prove failed error
    pub fn prove_failed(context: impl Into<String>) -> Self {
        NovaError::ProveFailed {
            context: context.into(),
        }
    }

    /// Create a compression failed error
    pub fn compression_failed(context: impl Into<String>) -> Self {
        NovaError::CompressionFailed {
            context: context.into(),
        }
    }

    /// Create a verification failed error
    pub fn verification_failed(context: impl Into<String>) -> Self {
        NovaError::VerificationFailed {
            context: context.into(),
        }
    }

    /// Create an invalid proof data error
    pub fn invalid_proof_data(context: impl Into<String>) -> Self {
        NovaError::InvalidProofData {
            context: context.into(),
        }
    }

    /// Create a state mismatch error
    pub fn state_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        NovaError::StateMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create an overflow error
    pub fn overflow(context: impl Into<String>) -> Self {
        NovaError::Overflow {
            context: context.into(),
        }
    }
}

/// Result type for Nova operations
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