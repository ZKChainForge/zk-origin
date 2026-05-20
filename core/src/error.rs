use thiserror::Error;

/// Core error type with detailed context
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid state error
    #[error("Invalid state: {context}")]
    InvalidState {
        /// Error context
        context: String
    },

    /// State hash mismatch error
    #[error("State hash mismatch: expected {expected}, got {actual}")]
    StateHashMismatch {
        /// Expected hash
        expected: String,
        /// Actual hash
        actual: String
    },

    /// State not found error
    #[error("State not found: {state_hash}")]
    StateNotFound {
        /// State hash that was not found
        state_hash: String
    },

    /// Invalid transition error
    #[error("Invalid transition: {context}")]
    InvalidTransition {
        /// Error context
        context: String
    },

    /// Invalid nonce error
    #[error("Nonce error: {context}")]
    InvalidNonce {
        /// Error context
        context: String
    },

    /// Invalid timestamp error
    #[error("Timestamp error: {context}")]
    InvalidTimestamp {
        /// Error context
        context: String
    },

    /// State difference check failed
    #[error("State difference check failed: states are identical")]
    StateDifferenceFailed,

    /// Policy violation error
    #[error("Policy violation: {context}")]
    PolicyViolation {
        /// Error context
        context: String
    },

    /// Authorization failed error
    #[error("Authorization failed: {context}")]
    AuthorizationFailed {
        /// Error context
        context: String
    },

    /// Rate limit exceeded error
    #[error("Rate limit exceeded: {origin_class} in epoch {epoch}")]
    RateLimitExceeded {
        /// Origin class that exceeded limit
        origin_class: String,
        /// Epoch number
        epoch: u64
    },

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid origin class error
    #[error("Invalid origin class: {context}")]
    InvalidOriginClass {
        /// Error context
        context: String
    },

    /// Generic error
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