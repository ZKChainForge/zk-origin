#![warn(missing_docs)]

//! ZK-ORIGIN Prover
//!
//! Witness generation and proof creation

/// Witness generation
pub mod witness;
/// Hashing utilities
pub mod hash;
/// Groth16 proof system
pub mod groth16;
/// Error types
pub mod error;
/// Utility functions
pub mod utils;

pub use witness::WitnessGenerator;
pub use error::{Error, Result};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}