/**
 * @title Nova IVC Module (PRODUCTION)
 * @notice Wrapper around Nova recursive SNARK implementation
 */

pub mod nova_ivc;
pub mod compression;
pub mod verification;

pub use nova_ivc::{NovaIVCProver, CompressedNovaProof, NovaError};
pub use compression::NovaCompressor;
pub use verification::NovaVerifier;

use std::marker::PhantomData;

/// Nova IVC configuration
pub struct NovaConfig {
    /// Number of steps before compression
    pub compression_threshold: usize,
    
    /// Whether to output Groth16 proof
    pub groth16_compression: bool,
}

impl Default for NovaConfig {
    fn default() -> Self {
        NovaConfig {
            compression_threshold: 100,
            groth16_compression: false,
        }
    }
}