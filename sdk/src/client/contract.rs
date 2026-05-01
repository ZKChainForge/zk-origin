//! Contract client for blockchain interactions

use crate::config::Config;
use crate::types::*;
use crate::error::{Error, Result};
use reqwest::Client;

/// Contract client
pub struct ContractClient {
    config: Config,
    client: Client,
    initialized: bool,
}

impl ContractClient {
    /// Create new contract client
    pub fn new(config: Config) -> Self {
        ContractClient {
            config,
            client: Client::new(),
            initialized: false,
        }
    }
    
    /// Initialize
    pub async fn initialize(&mut self) -> Result<()> {
        // Test RPC connection
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1,
        });
        
        let response = self.client
            .post(&self.config.rpc_endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::RpcError(format!("Failed to connect to RPC: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::RpcError("RPC returned error".to_string()));
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Submit proof
    pub async fn submit_proof(&self, proof: &Proof) -> Result<String> {
        if !self.initialized {
            return Err(Error::ContractError("Client not initialized".to_string()));
        }
        
        // Format proof for contract
        let formatted = self.format_proof(proof)?;
        
        // Create transaction
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendTransaction",
            "params": [{
                "to": self.config.contract_address,
                "data": formatted,
                "gas": "0x55f0a0",
            }],
            "id": 1,
        });
        
        let response = self.client
            .post(&self.config.rpc_endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::RpcError(format!("Transaction failed: {}", e)))?;
        
        let data: serde_json::Value = response.json().await
            .map_err(|e| Error::RpcError(format!("Failed to parse response: {}", e)))?;
        
        let tx_hash = data["result"]
            .as_str()
            .ok_or_else(|| Error::ContractError("No transaction hash".to_string()))?;
        
        Ok(tx_hash.to_string())
    }
    
    /// Get contract stats
    pub async fn get_stats(&self) -> Result<ContractStats> {
        if !self.initialized {
            return Err(Error::ContractError("Client not initialized".to_string()));
        }
        
        // TODO: Call contract view function
        Ok(ContractStats {
            total_transitions: 0,
            max_depth: 0,
            genesis_initialized: true,
            paused: false,
            current_epoch: 0,
        })
    }
    
    fn format_proof(&self, proof: &Proof) -> Result<String> {
        // TODO: Format proof as contract function call
        Ok(String::new())
    }
}