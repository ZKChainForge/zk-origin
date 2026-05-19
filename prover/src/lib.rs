#![warn(missing_docs)]
//! ZK-ORIGIN Prover Library
//!
//! Provides witness generation and proof generation capabilities for ZK-ORIGIN

/// Error types for the prover
pub mod error;
/// Cryptographic hash utilities
pub mod hash;
/// Witness generation and management
pub mod witness;

pub use error::{ProverError, Result};
pub use hash::Hash;
pub use witness::generator::WitnessGenerator;
pub use witness::{PrivateWitness, PublicWitness, TransitionWitness};

/// Current version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Module for production constants
pub mod consts {
    /// Number of origin classes
    pub const NUM_ORIGIN_CLASSES: usize = 7;

    /// Genesis origin class constant
    pub const ORIGIN_GENESIS: u8 = 255;

    /// Maximum lineage depth before requiring compression
    pub const MAX_LINEAGE_DEPTH: u32 = 1_000_000;

    /// Typical witness size in bytes
    pub const TYPICAL_WITNESS_SIZE: usize = 8192;

    /// Rate limits per origin class per epoch
    pub const RATE_LIMITS: [u32; 7] = [1, u32::MAX, 10, 100, 5, 1000, 1];
}