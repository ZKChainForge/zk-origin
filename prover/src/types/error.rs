//! Error types for ZK-ORIGIN

use std::fmt;
use thiserror::Error;

/// Main error type for ZK-ORIGIN operations
#[derive(Debug, Error)]
pub enum ZkOriginError {
    /// Policy violation - transition not allowed
    #[error("Policy violation: transition from {from} to {to} is not allowed")]
    PolicyViolation {
        /// Source origin class
        from: String,
        /// Target origin class
        to: String,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded for {origin}: {current}/{limit} in epoch {epoch}")]
    RateLimitExceeded {
        /// Origin class that exceeded limit
        origin: String,
        /// Current count
        current: u32,
        /// Maximum limit
        limit: u32,
        /// Epoch ID
        epoch: u64,
    },

    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    /// Invalid lineage
    #[error("Invalid lineage: {0}")]
    InvalidLineage(String),

    /// Genesis mismatch
    #[error("Genesis commitment mismatch")]
    GenesisMismatch,

    /// Not initialized
    #[error("Not initialized: {0}")]
    NotInitialized(String),

    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Invalid proof
    #[error("Invalid proof: {0}")]
    InvalidProof(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Proving error
    #[error("Proving error: {0}")]
    ProvingError(String),

    /// Circuit error
    #[error("Circuit error: {0}")]
    CircuitError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, ZkOriginError>;

// Helper constructors
impl ZkOriginError {
    /// Create a policy violation error
    pub fn policy_violation(from: impl fmt::Display, to: impl fmt::Display) -> Self {
        Self::PolicyViolation {
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit(origin: impl fmt::Display, current: u32, limit: u32, epoch: u64) -> Self {
        Self::RateLimitExceeded {
            origin: origin.to_string(),
            current,
            limit,
            epoch,
        }
    }

    /// Create a proving error
    pub fn proving(msg: impl fmt::Display) -> Self {
        Self::ProvingError(msg.to_string())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl fmt::Display) -> Self {
        Self::SerializationError(msg.to_string())
    }

    /// Create a deserialization error
    pub fn deserialization(msg: impl fmt::Display) -> Self {
        Self::DeserializationError(msg.to_string())
    }
}

/// Convert serde_json::Error to ZkOriginError for `?` usage
impl From<serde_json::Error> for ZkOriginError {
    fn from(err: serde_json::Error) -> Self {
        ZkOriginError::SerializationError(err.to_string())
    }
}

/// Optionally, also convert bincode errors if your crate uses them
impl From<bincode::Error> for ZkOriginError {
    fn from(err: bincode::Error) -> Self {
        ZkOriginError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ZkOriginError::policy_violation("User", "Admin");
        assert!(err.to_string().contains("User"));
        assert!(err.to_string().contains("Admin"));
    }

    #[test]
    fn test_proving_error() {
        let err = ZkOriginError::proving("test error");
        assert!(matches!(err, ZkOriginError::ProvingError(_)));
    }

    #[test]
    fn test_serde_from() {
        let json_err: serde_json::Error = serde_json::from_str::<u32>("bad").unwrap_err();
        let zk_err: ZkOriginError = json_err.into();
        match zk_err {
            ZkOriginError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }
}
