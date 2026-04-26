//! Error types

use thiserror::Error;

/// Core error type
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    /// Invalid transition
    #[error("Invalid transition: {0}")]
    InvalidTransition(String),
    
    /// Invalid nonce
    #[error("Invalid nonce: {0}")]
    InvalidNonce(String),
    
    /// Invalid timestamp
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    
    /// Policy violation
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    
    /// Authorization failed
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
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

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}