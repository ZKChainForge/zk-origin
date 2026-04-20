//! Batch Proof Generator
//!
//! Generate and submit multiple proofs in a single transaction
//! for efficiency and atomicity

use crate::groth16::prover::Groth16Prover;
use crate::witness::generator::WitnessGenerator;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchProof {
    pub proofs: Vec<SingleProof>,
    pub batch_id: String,
    pub timestamp: u64,
    pub total_time_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SingleProof {
    pub step: usize,
    pub proof: String,  // Serialized Groth16 proof
    pub public_inputs: Vec<String>,
    pub generation_time_ms: u64,
}

pub struct BatchProver {
    witness_generator: WitnessGenerator,
    groth16_prover: Groth16Prover,
    batch_size: usize,
    queue: VecDeque<WitnessData>,
}

#[derive(Clone, Debug)]
struct WitnessData {
    step: usize,
    prev_state_hash: [u8; 32],
    new_state_hash: [u8; 32],
    prev_origin_class: u8,
    new_origin_class: u8,
    prev_lineage_commitment: [u8; 32],
    prev_counter_commitment: [u8; 32],
    prev_counters: Vec<u32>,
    prev_depth: u32,
    epoch_id: u32,
    nonce: u64,
    prev_nonce: u64,
    timestamp: u64,
    prev_timestamp: u64,
}

impl BatchProver {
    pub fn new(
        witness_generator: WitnessGenerator,
        groth16_prover: Groth16Prover,
        batch_size: usize,
    ) -> Self {
        BatchProver {
            witness_generator,
            groth16_prover,
            batch_size,
            queue: VecDeque::new(),
        }
    }
    
    /// Add witness data to batch queue
    pub fn add_witness(&mut self, data: WitnessData) {
        self.queue.push_back(data);
    }
    
    /// Generate batch proofs when queue reaches batch size
    pub async fn generate_if_ready(&mut self) -> Result<Option<BatchProof>, String> {
        if self.queue.len() < self.batch_size {
            return Ok(None);
        }
        
        self.generate_batch().await.map(Some)
    }
    
    /// Force generate proofs for remaining items
    pub async fn flush(&mut self) -> Result<Option<BatchProof>, String> {
        if self.queue.is_empty() {
            return Ok(None);
        }
        
        self.generate_batch().await.map(Some)
    }
    
    async fn generate_batch(&mut self) -> Result<BatchProof, String> {
        let mut proofs = Vec::new();
        let batch_start = std::time::Instant::now();
        let batch_id = uuid::Uuid::new_v4().to_string();
        
        while let Some(witness_data) = self.queue.pop_front() {
            let step_start = std::time::Instant::now();
            
            // Generate witness
            let witness = self.witness_generator.generate(
                witness_data.prev_state_hash,
                witness_data.new_state_hash,
                witness_data.prev_origin_class,
                witness_data.new_origin_class,
                witness_data.prev_lineage_commitment,
                witness_data.prev_counter_commitment,
                witness_data.prev_counters,
                witness_data.prev_depth,
                witness_data.epoch_id,
                witness_data.nonce,
                witness_data.prev_nonce,
                witness_data.timestamp,
                witness_data.prev_timestamp,
                vec![],  // policy_merkle_proof
                vec![],  // policy_indices
            )?;
            
            // Generate proof
            let proof = self.groth16_prover.prove(&witness).await?;
            
            let generation_time_ms = step_start.elapsed().as_millis() as u64;
            
            proofs.push(SingleProof {
                step: witness_data.step,
                proof: serde_json::to_string(&proof)
                    .map_err(|e| e.to_string())?,
                public_inputs: vec![
                    witness.public.new_lineage_commitment,
                    witness.public.new_counter_commitment,
                    witness.public.lineage_valid.to_string(),
                ],
                generation_time_ms,
            });
        }
        
        let total_time_ms = batch_start.elapsed().as_millis() as u64;
        
        Ok(BatchProof {
            proofs,
            batch_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_time_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_batch_proof_generation() {

    }
}