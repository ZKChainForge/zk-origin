//! Commitment-based prover (fast but NOT zero-knowledge)
//!
//! This is for development and testing only.

use std::time::Instant;

use crate::hash::poseidon_native::NativePoseidonHasher;
use crate::types::lineage::CounterCommitment;
use crate::types::proof::ProofMetadata;
use crate::types::{LineageCommitment, LineageProof, StepWitness};
use crate::{Result, ZkOriginError};

/// Parameters for commitment-based proving
#[derive(Clone)]
pub struct CommitmentParams {
    /// Policy root hash
    pub policy_root: [u8; 32],
}

impl CommitmentParams {
    /// Create new commitment parameters
    pub fn new(policy_root: [u8; 32]) -> Self {
        Self { policy_root }
    }
}

/// Commitment-based prover
///
/// WARNING: This is NOT zero-knowledge!
pub struct CommitmentProver {
    params: CommitmentParams,
    hasher: NativePoseidonHasher,
    genesis_commitment: [u8; 32],
    current_lineage: [u8; 32],
    current_counters: [u8; 32],
    num_steps: usize,
    proof_data: Vec<u8>,
    total_time_us: u64,
}

impl CommitmentProver {
    /// Create a new commitment prover
    pub fn new(params: CommitmentParams) -> Self {
        Self {
            params,
            hasher: NativePoseidonHasher::new(),
            genesis_commitment: [0u8; 32],
            current_lineage: [0u8; 32],
            current_counters: [0u8; 32],
            num_steps: 0,
            proof_data: Vec::new(),
            total_time_us: 0,
        }
    }

    /// Initialize with genesis state
    pub fn initialize(&mut self, genesis_state_hash: [u8; 32], epoch: u64) -> Result<()> {
        let genesis_lineage = self.hasher.compute_genesis_commitment(&genesis_state_hash);
        let initial_counters = self.hasher.compute_counter_commitment(epoch, &[0; 6]);

        self.genesis_commitment = genesis_lineage;
        self.current_lineage = genesis_lineage;
        self.current_counters = initial_counters;
        self.num_steps = 0;
        self.proof_data.clear();
        self.total_time_us = 0;

        Ok(())
    }

    /// Add a step
    pub fn add_step(&mut self, witness: &StepWitness) -> Result<()> {
        let start = Instant::now();

        let transition_hash = self.hasher.compute_transition_hash(
            &witness.prev_state_hash,
            &witness.new_state_hash,
            witness.new_origin as u8,
            witness.timestamp,
            witness.epoch_id,
        );

        self.current_lineage = self.hasher.compute_lineage_commitment(
            &self.current_lineage,
            &transition_hash,
            self.num_steps as u64 + 1,
        );

        let new_counters = witness.compute_new_counters();
        self.current_counters = self
            .hasher
            .compute_counter_commitment(witness.epoch_id, &new_counters);

        self.proof_data.extend_from_slice(&transition_hash);

        self.num_steps += 1;
        self.total_time_us += start.elapsed().as_micros() as u64;

        Ok(())
    }

    /// Finalize and create proof
    pub fn finalize(&self) -> Result<LineageProof> {
        if self.num_steps == 0 {
            return Err(ZkOriginError::InvalidLineage("No steps to prove".into()));
        }

        let start = Instant::now();
        let proof_bytes = self.create_commitment_proof();
        let finalize_time = start.elapsed().as_micros() as u64;

        let metadata = ProofMetadata::new()
            .with_proving_time((self.total_time_us + finalize_time) / 1000)
            .with_notes("COMMITMENT MODE - NOT ZERO-KNOWLEDGE".to_string());

        let proof = LineageProof::new(
            proof_bytes,
            LineageCommitment::new(self.current_lineage, self.num_steps as u64),
            CounterCommitment::new(self.current_counters, 0),
            LineageCommitment::new(self.genesis_commitment, 0),
            self.num_steps as u64,
            self.params.policy_root,
        )
        .with_metadata(metadata);

        Ok(proof)
    }

    fn create_commitment_proof(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(b"zk-origin-commitment-v1");
        hasher.update(self.genesis_commitment);
        hasher.update(self.current_lineage);
        hasher.update(self.current_counters);
        hasher.update((self.num_steps as u64).to_le_bytes());
        hasher.update(&self.proof_data);

        hasher.finalize().to_vec()
    }

    /// Get current depth
    pub fn current_depth(&self) -> usize {
        self.num_steps
    }

    /// Get genesis
    pub fn genesis(&self) -> &[u8; 32] {
        &self.genesis_commitment
    }

    /// Get current lineage
    pub fn current_lineage(&self) -> &[u8; 32] {
        &self.current_lineage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prover::WitnessGenerator;
    use crate::types::{OriginClass, OriginPolicy, Transition};

    #[test]
    fn test_commitment_prover() {
        let policy = OriginPolicy::default();
        let params = CommitmentParams::new(policy.compute_hash());

        let mut prover = CommitmentProver::new(params);
        prover.initialize([0u8; 32], 0).unwrap();

        let mut witness_gen = WitnessGenerator::new(policy);
        witness_gen.reset([0u8; 32]);

        let transition = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);

        let witness = witness_gen.generate_witness(&transition).unwrap();
        prover.add_step(&witness).unwrap();

        assert_eq!(prover.current_depth(), 1);

        let proof = prover.finalize().unwrap();
        assert!(proof.proof_size() > 0);
        assert!(!proof.is_real_zk());
    }
}
