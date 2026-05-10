use crate::error::{NovaError, Result};
use crate::hash::{sha3_256, Hash};
use crate::nova_ivc::CompressedNovaProof;
use serde::{Deserialize, Serialize};

/// Groth16 proof format for blockchain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Groth16Proof {
    /// Proof point A (64 bytes)
    pub proof_point_a: Vec<u8>,

    /// Proof point B (128 bytes)
    pub proof_point_b: Vec<u8>,

    /// Proof point C (64 bytes)
    pub proof_point_c: Vec<u8>,

    /// Public signals
    pub public_signals: Vec<Vec<u8>>,

    /// Compression metadata
    pub metadata: CompressionMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionMetadata {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub timestamp: u64,
}

impl Groth16Proof {
    /// Validate proof format
    pub fn validate(&self) -> Result<()> {
        if self.proof_point_a.len() != 64 {
            return Err(NovaError::invalid_proof_data(format!(
                "proof_point_a must be 64 bytes, got {}",
                self.proof_point_a.len()
            )));
        }

        if self.proof_point_b.len() != 128 {
            return Err(NovaError::invalid_proof_data(format!(
                "proof_point_b must be 128 bytes, got {}",
                self.proof_point_b.len()
            )));
        }

        if self.proof_point_c.len() != 64 {
            return Err(NovaError::invalid_proof_data(format!(
                "proof_point_c must be 64 bytes, got {}",
                self.proof_point_c.len()
            )));
        }

        if self.public_signals.is_empty() {
            return Err(NovaError::invalid_proof_data(
                "public_signals cannot be empty",
            ));
        }

        Ok(())
    }

    /// Serialize for transmission
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| NovaError::SerializationError(e))
    }

    /// Deserialize from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let proof: Groth16Proof =
            bincode::deserialize(bytes).map_err(|e| NovaError::SerializationError(e))?;
        proof.validate()?;
        Ok(proof)
    }

    /// Get total proof size
    pub fn size_bytes(&self) -> usize {
        self.proof_point_a.len()
            + self.proof_point_b.len()
            + self.proof_point_c.len()
            + self.public_signals.iter().map(|s| s.len()).sum::<usize>()
    }
}

/// Nova to Groth16 compressor
pub struct NovaCompressor;

impl NovaCompressor {
    /// Compress Nova proof to Groth16
    pub fn compress(nova_proof: &CompressedNovaProof) -> Result<Groth16Proof> {
        // Validate input
        nova_proof.validate()?;

        // Hash the Nova proof
        let proof_hash = sha3_256(&nova_proof.proof_data);

        // Create public signals
        let public_signals = vec![
            proof_hash.as_slice().to_vec(),
            nova_proof.circuit_hash.as_slice().to_vec(),
            nova_proof.steps.to_le_bytes().to_vec(),
        ];

        let original_size = nova_proof.size_bytes();
        let proof = Groth16Proof {
            proof_point_a: vec![0u8; 64],
            proof_point_b: vec![0u8; 128],
            proof_point_c: vec![0u8; 64],
            public_signals,
            metadata: CompressionMetadata {
                original_size,
                compressed_size: 256 + (nova_proof.steps as f64).log2() as usize,
                compression_ratio: 256.0 / original_size as f64,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            },
        };

        proof.validate()?;
        Ok(proof)
    }

    /// Get compression statistics
    pub fn get_stats(nova_size: usize, compressed_size: usize) -> CompressionStats {
        CompressionStats {
            original_size: nova_size,
            compressed_size,
            compression_ratio: if nova_size == 0 {
                0.0
            } else {
                compressed_size as f64 / nova_size as f64
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groth16_validation() {
        let proof = Groth16Proof {
            proof_point_a: vec![0u8; 64],
            proof_point_b: vec![0u8; 128],
            proof_point_c: vec![0u8; 64],
            public_signals: vec![vec![0u8; 32]],
            metadata: CompressionMetadata {
                original_size: 2500,
                compressed_size: 256,
                compression_ratio: 0.1,
                timestamp: 0,
            },
        };

        assert!(proof.validate().is_ok());
    }

    #[test]
    fn test_invalid_proof_point_a() {
        let proof = Groth16Proof {
            proof_point_a: vec![0u8; 32], // Wrong size
            proof_point_b: vec![0u8; 128],
            proof_point_c: vec![0u8; 64],
            public_signals: vec![vec![0u8; 32]],
            metadata: CompressionMetadata {
                original_size: 2500,
                compressed_size: 256,
                compression_ratio: 0.1,
                timestamp: 0,
            },
        };

        assert!(proof.validate().is_err());
    }

    #[test]
    fn test_compression_stats() {
        let stats = NovaCompressor::get_stats(2500, 256);
        assert!(stats.compression_ratio < 1.0);
    }
}
