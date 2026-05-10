use std::fmt;
use thiserror::Error;

/// Core error type with detailed context
#[derive(Error, Debug)]
pub enum Error {
    // State errors
    #[error("Invalid state: {context}")]
    InvalidState { context: String },

    #[error("State hash mismatch: expected {expected}, got {actual}")]
    StateHashMismatch { expected: String, actual: String },

    #[error("State not found: {state_hash}")]
    StateNotFound { state_hash: String },

    // Transition errors
    #[error("Invalid transition: {context}")]
    InvalidTransition { context: String },

    #[error("Nonce error: {context}")]
    InvalidNonce { context: String },

    #[error("Timestamp error: {context}")]
    InvalidTimestamp { context: String },

    #[error("State difference check failed: states are identical")]
    StateDifferenceFailed,

    // Policy and authorization
    #[error("Policy violation: {context}")]
    PolicyViolation { context: String },

    #[error("Authorization failed: {context}")]
    AuthorizationFailed { context: String },

    #[error("Rate limit exceeded: {origin_class} in epoch {epoch}")]
    RateLimitExceeded { origin_class: String, epoch: u64 },

    // Serialization
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    // IO
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    // Origin detection
    #[error("Invalid origin class: {context}")]
    InvalidOriginClass { context: String },

    // Generic
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create InvalidState error
    pub fn invalid_state(context: impl Into<String>) -> Self {
        Error::InvalidState {
            context: context.into(),
        }
    }

    /// Create InvalidTransition error
    pub fn invalid_transition(context: impl Into<String>) -> Self {
        Error::InvalidTransition {
            context: context.into(),
        }
    }

    /// Create InvalidNonce error
    pub fn invalid_nonce(context: impl Into<String>) -> Self {
        Error::InvalidNonce {
            context: context.into(),
        }
    }

    /// Create InvalidTimestamp error
    pub fn invalid_timestamp(context: impl Into<String>) -> Self {
        Error::InvalidTimestamp {
            context: context.into(),
        }
    }

    /// Create PolicyViolation error
    pub fn policy_violation(context: impl Into<String>) -> Self {
        Error::PolicyViolation {
            context: context.into(),
        }
    }

    /// Create AuthorizationFailed error
    pub fn authorization_failed(context: impl Into<String>) -> Self {
        Error::AuthorizationFailed {
            context: context.into(),
        }
    }

    /// Create RateLimitExceeded error
    pub fn rate_limit_exceeded(origin_class: impl Into<String>, epoch: u64) -> Self {
        Error::RateLimitExceeded {
            origin_class: origin_class.into(),
            epoch,
        }
    }

    /// Create InvalidOriginClass error
    pub fn invalid_origin_class(context: impl Into<String>) -> Self {
        Error::InvalidOriginClass {
            context: context.into(),
        }
    }
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::invalid_state("test");
        assert!(err.to_string().contains("Invalid state"));
    }

    #[test]
    fn test_error_context() {
        let err = Error::rate_limit_exceeded("User", 42);
        assert!(err.to_string().contains("epoch 42"));
    }
}
