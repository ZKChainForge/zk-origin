
use crate::groth16::prover::Groth16Prover;
use crate::witness::generator::WitnessGenerator;
use crate::error::Result;
use crate::Hash;
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
    pub proof: String,
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
pub struct WitnessData {
    pub step: usize,
    pub prev_state_hash: Hash,
    pub new_state_hash: Hash,
    pub prev_origin_class: u8,
    pub new_origin_class: u8,
    pub prev_lineage_commitment: Hash,
    pub prev_counter_commitment: Hash,
    pub prev_counters: Vec<u32>,
    pub prev_depth: u32,
    pub epoch_id: u32,
    pub nonce: u64,
    pub prev_nonce: u64,
    pub timestamp: u64,
    pub prev_timestamp: u64,
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
    
    pub fn add_witness(&mut self, data: WitnessData) {
        self.queue.push_back(data);
    }
    
    pub fn generate_if_ready(&mut self) -> Result<Option<BatchProof>> {
        if self.queue.len() < self.batch_size {
            return Ok(None);
        }
        
        self.generate_batch().map(Some)
    }
    
    pub fn flush(&mut self) -> Result<Option<BatchProof>> {
        if self.queue.is_empty() {
            return Ok(None);
        }
        
        self.generate_batch().map(Some)
    }
    
    fn generate_batch(&mut self) -> Result<BatchProof> {
        let mut proofs = Vec::new();
        let batch_start = std::time::Instant::now();
        let batch_id = uuid::Uuid::new_v4().to_string();
        
        while let Some(witness_data) = self.queue.pop_front() {
            let step_start = std::time::Instant::now();
            
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
                vec![],
                vec![],
            )?;
            
            let proof = self.groth16_prover.prove(&witness)?;
            
            let generation_time_ms = step_start.elapsed().as_millis() as u64;
            
            proofs.push(SingleProof {
                step: witness_data.step,
                proof: serde_json::to_string(&proof)
                    .map_err(|e| crate::error::ProverError::SerializationError(e.to_string()))?,
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