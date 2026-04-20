//! ZK-ORIGIN Prover
//!
//! Witness generation and proof creation

#![warn(missing_docs)]

pub mod witness;
pub mod hash;
pub mod groth16;
pub mod error;
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