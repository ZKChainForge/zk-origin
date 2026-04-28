/**
 * @title ZK-ORIGIN Nova Module (PRODUCTION STUB)
 * @notice Nova IVC proof generation and compression
 * 
 * ARCHITECTURE:
 * - IVC (Incrementally Verifiable Computation) for lineage folding
 * - Constant-size proofs independent of lineage depth
 * - Compression to Groth16 for blockchain verification
 * - No trusted setup required
 */

pub mod nova_ivc;
pub mod compression;
pub mod verification;

pub use nova_ivc::{
    NovaIVCProver,
    CompressedNovaProof,
    NovaError,
    NovaConfig,
};
pub use compression::NovaCompressor;
pub use verification::NovaVerifier;