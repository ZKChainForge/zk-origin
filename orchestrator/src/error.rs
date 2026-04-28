//! Error types

use thiserror::Error;

/// Orchestrator error type
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// RPC error
    #[error("RPC error: {0}")]
    RpcError(String),
    
    /// Execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
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