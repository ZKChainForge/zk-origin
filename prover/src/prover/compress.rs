//! Proof compression utilities

use crate::types::LineageProof;
use crate::{Result, ZkOriginError};

/// Compressed proof format
#[derive(Clone, Debug)]
pub struct CompressedProof {
    /// Compressed proof bytes
    pub bytes: Vec<u8>,
    
    /// Original proof size
    pub original_size: usize,
    
    /// Compression ratio
    pub ratio: f64,
}

impl CompressedProof {
    /// Compress a lineage proof
    pub fn compress(proof: &LineageProof) -> Result<Self> {
        // In production, this would use Nova's CompressedSNARK
        // For now, we just return the proof as-is
        
        let bytes = proof.to_bytes()?;
        let original_size = bytes.len();
        
        // Placeholder: no actual compression
        Ok(Self {
            bytes,
            original_size,
            ratio: 1.0,
        })
    }

    /// Decompress back to a proof
    pub fn decompress(&self) -> Result<LineageProof> {
        LineageProof::from_bytes(&self.bytes)
    }

    /// Get compressed size
    pub fn size(&self) -> usize {
        self.bytes.len()
    }
}

/// Compress multiple proofs into a batch
pub fn batch_compress(proofs: &[LineageProof]) -> Result<Vec<CompressedProof>> {
    proofs.iter().map(CompressedProof::compress).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LineageCommitment, CounterCommitment};

    fn create_test_proof() -> LineageProof {
        LineageProof::new(
            vec![1, 2, 3, 4, 5],
            LineageCommitment::new([1u8; 32], 10),
            CounterCommitment::new([2u8; 32], 0),
            LineageCommitment::genesis([0u8; 32]),
            10,
            [3u8; 32],
        )
    }

    #[test]
    fn test_compress_decompress() {
        let proof = create_test_proof();
        
        let compressed = CompressedProof::compress(&proof).unwrap();
        let decompressed = compressed.decompress().unwrap();
        
        assert_eq!(proof.num_steps, decompressed.num_steps);
        assert_eq!(proof.final_lineage.value, decompressed.final_lineage.value);
    }

    #[test]
    fn test_batch_compress() {
        let proofs: Vec<_> = (0..3).map(|_| create_test_proof()).collect();
        
        let compressed = batch_compress(&proofs).unwrap();
        
        assert_eq!(compressed.len(), 3);
    }
}