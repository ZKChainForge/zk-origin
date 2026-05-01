//! ZK-ORIGIN SDK
//!
//! High-level interface for interacting with ZK-ORIGIN system

#![warn(missing_docs)]

pub mod types;
pub mod client;
pub mod utils;
pub mod error;
pub mod config;

pub use client::{ProverClient, ContractClient, StateClient};
pub use config::Config;
pub use error::{Error, Result};
pub use types::*;

/// Main SDK struct
pub struct ZKOrigin {
    /// Prover client
    pub prover: ProverClient,
    /// Contract client
    pub contract: ContractClient,
    /// State client
    pub state: StateClient,
    /// Configuration
    config: Config,
}

impl ZKOrigin {
    /// Create new SDK instance
    pub fn new(config: Config) -> Self {
        ZKOrigin {
            prover: ProverClient::new(config.clone()),
            contract: ContractClient::new(config.clone()),
            state: StateClient::new(config.clone()),
            config,
        }
    }
    
    /// Initialize SDK
    pub async fn initialize(&mut self) -> Result<()> {
        self.prover.initialize().await?;
        self.contract.initialize().await?;
        self.state.initialize().await?;
        Ok(())
    }
    
    /// Generate and submit proof
    pub async fn submit_transition(
        &self,
        prev_state: &[u8],
        new_state: &[u8],
        origin_class: u8,
    ) -> Result<String> {
        // Generate witness
        let witness = self.prover.generate_witness(prev_state, new_state, origin_class).await?;
        
        // Generate proof
        let proof = self.prover.generate_proof(&witness).await?;
        
        // Submit to contract
        let tx_hash = self.contract.submit_proof(&proof).await?;
        
        Ok(tx_hash)
    }
    
    /// Verify state
    pub async fn verify_state(&self, state_hash: &[u8; 32]) -> Result<bool> {
        self.state.is_verified(state_hash).await
    }
    
    /// Get lineage
    pub async fn get_lineage(&self, state_hash: &[u8; 32]) -> Result<Lineage> {
        self.state.get_lineage(state_hash).await
    }
    
    /// Get contract stats
    pub async fn get_stats(&self) -> Result<ContractStats> {
        self.contract.get_stats().await
    }
    
    /// Get config
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sdk_creation() {
        let config = Config::default();
        let sdk = ZKOrigin::new(config);
        assert!(sdk.prover.is_initialized() == false);
    }
}