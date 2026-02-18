//! Error types for ZK-ORIGIN

use thiserror::Error;
use std::fmt;

/// Main error type for ZK-ORIGIN operations
#[derive(Error, Debug)]
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
    #[error("Rate limit exceeded for {origin_class}: {current}/{limit} in epoch {epoch}")]
    RateLimitExceeded {
        /// The origin class that hit the limit
        origin_class: String,
        /// Current count
        current: u32,
        /// The limit
        limit: u32,
        /// Current epoch
        epoch: u64,
    },

    /// Invalid lineage - state doesn't match expected
    #[error("Invalid lineage: {0}")]
    InvalidLineage(String),

    /// Invalid state hash
    #[error("Invalid state hash: {0}")]
    InvalidStateHash(String),

    /// Witness generation failed
    #[error("Witness generation failed: {0}")]
    WitnessGenerationFailed(String),

    /// Proving failed
    #[error("Proving failed: {0}")]
    ProvingFailed(String),

    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Invalid proof
    #[error("Invalid proof: {0}")]
    InvalidProof(String),

    /// Circuit error
    #[error("Circuit error: {0}")]
    CircuitError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Not initialized
    #[error("Prover not initialized: {0}")]
    NotInitialized(String),

    /// Epoch mismatch
    #[error("Epoch mismatch: expected {expected}, got {actual}")]
    EpochMismatch {
        /// Expected epoch
        expected: u64,
        /// Actual epoch
        actual: u64,
    },

    /// Genesis mismatch
    #[error("Genesis mismatch: proof genesis doesn't match expected")]
    GenesisMismatch,

    /// Depth overflow
    #[error("Lineage depth overflow: {0} exceeds maximum")]
    DepthOverflow(u64),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

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
            origin_class: origin.to_string(),
            current,
            limit,
            epoch,
        }
    }

    /// Check if this is a policy-related error
    pub fn is_policy_error(&self) -> bool {
        matches!(self, Self::PolicyViolation { .. } | Self::RateLimitExceeded { .. })
    }

    /// Check if this is a verification error
    pub fn is_verification_error(&self) -> bool {
        matches!(self, Self::VerificationFailed(_) | Self::InvalidProof(_))
    }

    /// Get error code for programmatic handling
    pub fn code(&self) -> &'static str {
        match self {
            Self::PolicyViolation { .. } => "POLICY_VIOLATION",
            Self::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            Self::InvalidLineage(_) => "INVALID_LINEAGE",
            Self::InvalidStateHash(_) => "INVALID_STATE_HASH",
            Self::WitnessGenerationFailed(_) => "WITNESS_GEN_FAILED",
            Self::ProvingFailed(_) => "PROVING_FAILED",
            Self::VerificationFailed(_) => "VERIFICATION_FAILED",
            Self::InvalidProof(_) => "INVALID_PROOF",
            Self::CircuitError(_) => "CIRCUIT_ERROR",
            Self::SerializationError(_) => "SERIALIZATION_ERROR",
            Self::ConfigurationError(_) => "CONFIG_ERROR",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::NotInitialized(_) => "NOT_INITIALIZED",
            Self::EpochMismatch { .. } => "EPOCH_MISMATCH",
            Self::GenesisMismatch => "GENESIS_MISMATCH",
            Self::DepthOverflow(_) => "DEPTH_OVERFLOW",
            Self::IoError(_) => "IO_ERROR",
        }
    }
}

/// Result type alias for ZK-ORIGIN operations
pub type Result<T> = std::result::Result<T, ZkOriginError>;

/// Extension trait for Results
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, msg: impl Into<String>) -> Result<T>;
}

impl<T, E: std::error::Error> ResultExt<T> for std::result::Result<T, E> {
    fn context(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| ZkOriginError::InternalError(format!("{}: {}", msg.into(), e)))
    }
}

impl From<bincode::Error> for ZkOriginError {
    fn from(e: bincode::Error) -> Self {
        ZkOriginError::SerializationError(e.to_string())
    }
}

impl From<serde_json::Error> for ZkOriginError {
    fn from(e: serde_json::Error) -> Self {
        ZkOriginError::SerializationError(e.to_string())
    }
}

impl From<hex::FromHexError> for ZkOriginError {
    fn from(e: hex::FromHexError) -> Self {
        ZkOriginError::SerializationError(format!("Hex decode error: {}", e))
    }
}

/// Macro for creating internal errors with location info
#[macro_export]
macro_rules! internal_error {
    ($msg:expr) => {
        $crate::ZkOriginError::InternalError(format!(
            "{} at {}:{}",
            $msg,
            file!(),
            line!()
        ))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ZkOriginError::InternalError(format!(
            "{} at {}:{}",
            format!($fmt, $($arg)*),
            file!(),
            line!()
        ))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_violation_error() {
        let err = ZkOriginError::policy_violation("User", "Admin");
        assert!(err.is_policy_error());
        assert_eq!(err.code(), "POLICY_VIOLATION");
        assert!(err.to_string().contains("User"));
        assert!(err.to_string().contains("Admin"));
    }

    #[test]
    fn test_rate_limit_error() {
        let err = ZkOriginError::rate_limit("Admin", 10, 10, 42);
        assert!(err.is_policy_error());
        assert_eq!(err.code(), "RATE_LIMIT_EXCEEDED");
    }

    #[test]
    fn test_error_codes() {
        let errors = vec![
            ZkOriginError::InvalidLineage("test".into()),
            ZkOriginError::ProvingFailed("test".into()),
            ZkOriginError::GenesisMismatch,
        ];

        for err in errors {
            assert!(!err.code().is_empty());
        }
    }

    #[test]
    fn test_internal_error_macro() {
        let err = internal_error!("Something went wrong");
        assert!(err.to_string().contains("Something went wrong"));
        assert!(err.to_string().contains("error.rs"));
    }
}