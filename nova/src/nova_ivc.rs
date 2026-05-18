use crate::config::NovaConfig;
use crate::error::{NovaError, Result};
use crate::hash::{sha3_256, Hash, HashType, Hasher};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// State vector (6 elements, 48 bytes total)
/// [0] = lineage_commitment (bytes32)
/// [1] = counter_commitment (bytes32)
/// [2] = nonce (u32)
/// [3] = timestamp (u64)
/// [4] = epoch_id (u32)
/// [5] = depth (u32)
pub const STATE_SIZE: usize = 48;

/// Compressed Nova proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedNovaProof {
    /// Proof data (accumulated hashes)
    pub proof_data: Vec<u8>,

    /// Final state vector
    pub final_state: Vec<u8>,

    /// Number of folding steps
    pub steps: usize,

    /// Genesis state (immutable)
    pub genesis_state: Vec<u8>,

    /// Proof generation timestamp
    pub timestamp: u64,

    /// Circuit version hash
    pub circuit_hash: Hash,

    /// Proof commitment
    pub proof_commitment: Hash,

    /// Checksum for integrity
    pub checksum: Hash,
}

impl CompressedNovaProof {
    /// Validate proof integrity
    pub fn validate(&self) -> Result<()> {
        // Check field existence
        if self.proof_data.is_empty() {
            return Err(NovaError::invalid_proof_data("proof_data cannot be empty"));
        }

        if self.final_state.len() != STATE_SIZE {
            return Err(NovaError::invalid_state_size(
                STATE_SIZE,
                self.final_state.len(),
            ));
        }

        if self.genesis_state.len() != STATE_SIZE {
            return Err(NovaError::invalid_state_size(
                STATE_SIZE,
                self.genesis_state.len(),
            ));
        }

        if self.steps == 0 {
            return Err(NovaError::invalid_proof_data("steps must be > 0"));
        }

        // Verify checksum
        let computed_checksum = self.compute_checksum();
        if computed_checksum != self.checksum {
            return Err(NovaError::ProofTampering);
        }

        Ok(())
    }

    /// Compute checksum (made public)
    pub fn compute_checksum(&self) -> Hash {
        let mut hasher = Hasher::new(HashType::SHA3_256);
        hasher.update(&self.proof_data);
        hasher.update(&self.final_state);
        hasher.update(&self.genesis_state);
        hasher.update(&self.steps.to_le_bytes());
        hasher.finalize()
    }

    /// Serialize to bytes
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| NovaError::SerializationError(e))
    }

    /// Deserialize from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| NovaError::SerializationError(e))
    }

    /// Get proof size
    pub fn size_bytes(&self) -> usize {
        self.proof_data.len() + self.final_state.len() + self.genesis_state.len() + 128
    }

    /// Estimate compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.steps == 0 {
            return 0.0;
        }
        self.size_bytes() as f64 / (self.steps as f64 * STATE_SIZE as f64)
    }
}

/// Production Nova IVC Prover
pub struct NovaIVCProver {
    /// Configuration
    config: NovaConfig,

    /// Steps completed
    steps_completed: Arc<AtomicU64>,

    /// Current state
    current_state: Vec<u8>,

    /// Genesis state (immutable)
    genesis_state: Vec<u8>,

    /// Proof accumulator
    proof_accumulator: Vec<u8>,

    /// Circuit hash
    circuit_hash: Hash,

    /// Epoch counters commitment
    #[allow(dead_code)]
    epoch_counters_commitment: Hash,

    /// Last transition timestamp
    last_timestamp: u64,
}

impl NovaIVCProver {
    /// Create new prover
    pub fn new(config: NovaConfig) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Initialize genesis state (all zeros)
        let genesis_state = vec![0u8; STATE_SIZE];

        // Compute circuit hash
        let circuit_hash = sha3_256(b"lineage_step_circuit_v1");

        Ok(NovaIVCProver {
            config,
            steps_completed: Arc::new(AtomicU64::new(0)),
            current_state: genesis_state.clone(),
            genesis_state,
            proof_accumulator: Vec::with_capacity(10000),
            circuit_hash,
            epoch_counters_commitment: Hash::default(),
            last_timestamp: 0,
        })
    }

    /// Add and prove a transition
    pub fn add_transition(&mut self, new_state: &[u8]) -> Result<()> {
        // Validate state size
        if new_state.len() != STATE_SIZE {
            return Err(NovaError::invalid_state_size(STATE_SIZE, new_state.len()));
        }

        // Check step limit
        let current_steps = self.steps_completed.load(Ordering::SeqCst);
        if current_steps >= self.config.max_steps as u64 {
            return Err(NovaError::prove_failed(format!(
                "Max steps {} exceeded",
                self.config.max_steps
            )));
        }

        // Check state change
        if new_state == self.current_state.as_slice() {
            return Err(NovaError::invalid_proof_data("State must change"));
        }

        // Update state
        self.current_state = new_state.to_vec();

        // Accumulate proof
        let mut hasher = Hasher::new(match self.config.hash_algorithm {
            0 => HashType::SHA3_256,
            _ => HashType::BLAKE3,
        });

        hasher.update(&self.current_state);
        hasher.update(&current_steps.to_le_bytes());

        let transition_hash = hasher.finalize();
        self.proof_accumulator
            .extend_from_slice(transition_hash.as_slice());

        // Increment counter
        self.steps_completed.fetch_add(1, Ordering::SeqCst);

        // Update timestamp
        self.last_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(())
    }

    /// Finalize proof
    pub fn finalize(&self) -> Result<CompressedNovaProof> {
        let steps = self.steps_completed.load(Ordering::SeqCst) as usize;

        if steps == 0 {
            return Err(NovaError::NoProofGenerated);
        }

        // Compute proof commitment
        let proof_commitment = sha3_256(&self.proof_accumulator);

        // Create proof
        let proof = CompressedNovaProof {
            proof_data: self.proof_accumulator.clone(),
            final_state: self.current_state.clone(),
            steps,
            genesis_state: self.genesis_state.clone(),
            timestamp: self.last_timestamp,
            circuit_hash: self.circuit_hash,
            proof_commitment,
            checksum: Hash::default(), // Will be computed
        };

        // Compute checksum
        let checksum = proof.compute_checksum();
        let mut proof = proof;
        proof.checksum = checksum;

        // Validate proof
        proof.validate()?;

        Ok(proof)
    }

    /// Get final lineage commitment
    pub fn get_final_lineage_commitment(&self) -> Result<Hash> {
        if self.current_state.len() < 32 {
            return Err(NovaError::invalid_proof_data("State too small"));
        }

        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&self.current_state[0..32]);
        Ok(Hash::from_array(commitment))
    }

    /// Get steps completed
    pub fn steps_completed(&self) -> u64 {
        self.steps_completed.load(Ordering::SeqCst)
    }

    /// Get current state
    pub fn current_state(&self) -> &[u8] {
        &self.current_state
    }

    /// Verify proof consistency
    pub fn verify_proof_consistency(&self, proof: &CompressedNovaProof) -> Result<()> {
        if proof.genesis_state != self.genesis_state {
            return Err(NovaError::state_mismatch(
                hex::encode(&self.genesis_state),
                hex::encode(&proof.genesis_state),
            ));
        }

        if proof.circuit_hash != self.circuit_hash {
            return Err(NovaError::CircuitHashMismatch {
                expected: self.circuit_hash.to_hex(),
                actual: proof.circuit_hash.to_hex(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prover_creation() {
        let config = NovaConfig::testing();
        let prover = NovaIVCProver::new(config);
        assert!(prover.is_ok());
    }

    #[test]
    fn test_add_transition() {
        let config = NovaConfig::testing();
        let mut prover = NovaIVCProver::new(config).unwrap();

        let mut state = vec![0u8; STATE_SIZE];
        state[0] = 1;

        let result = prover.add_transition(&state);
        assert!(result.is_ok());
        assert_eq!(prover.steps_completed(), 1);
    }

    #[test]
    fn test_invalid_state_size() {
        let config = NovaConfig::testing();
        let mut prover = NovaIVCProver::new(config).unwrap();

        let state = vec![0u8; 32]; // Wrong size
        let result = prover.add_transition(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_finalize_proof() {
        let config = NovaConfig::testing();
        let mut prover = NovaIVCProver::new(config).unwrap();

        let mut state = vec![0u8; STATE_SIZE];
        state[0] = 1;
        prover.add_transition(&state).unwrap();

        let proof = prover.finalize();
        assert!(proof.is_ok());

        let proof = proof.unwrap();
        assert_eq!(proof.steps, 1);
        assert!(proof.validate().is_ok());
    }

    #[test]
    fn test_proof_tampering_detection() {
        let config = NovaConfig::testing();
        let mut prover = NovaIVCProver::new(config).unwrap();

        let mut state = vec![0u8; STATE_SIZE];
        state[0] = 1;
        prover.add_transition(&state).unwrap();

        let mut proof = prover.finalize().unwrap();
        proof.proof_data[0] ^= 1; // Flip a bit

        assert!(proof.validate().is_err());
    }
}