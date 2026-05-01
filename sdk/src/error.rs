//! Error types for SDK

use thiserror::Error;

/// SDK Error type
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// RPC error
    #[error("RPC error: {0}")]
    RpcError(String),
    
    /// Proof generation error
    #[error("Proof generation error: {0}")]
    ProofError(String),
    
    /// Witness generation error
    #[error("Witness generation error: {0}")]
    WitnessError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Contract error
    #[error("Contract error: {0}")]
    ContractError(String),
    
    /// State error
    #[error("State error: {0}")]
    StateError(String),
    
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