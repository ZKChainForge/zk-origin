//! ZK-ORIGIN Orchestrator
//!
//! Coordinates end-to-end proof generation and submission

#![warn(missing_docs)]

pub mod pipeline;
pub mod config;
pub mod rpc;
pub mod error;
pub mod metrics;

pub use pipeline::PipelineExecutor;
pub use config::Config;
pub use error::{Error, Result};
pub use metrics::Metrics;

/// Orchestrator version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}