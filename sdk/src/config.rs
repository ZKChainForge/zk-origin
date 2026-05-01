//! Configuration management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::error::{Error, Result};

/// SDK Configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Ethereum RPC endpoint
    pub rpc_endpoint: String,
    
    /// Contract address
    pub contract_address: String,
    
    /// Prover binary path
    pub prover_binary: String,
    
    /// Circuit path
    pub circuit_path: String,
    
    /// Policy root
    pub policy_root: String,
    
    /// Genesis hash
    pub genesis_hash: String,
    
    /// Private key (for signing)
    #[serde(skip)]
    pub private_key: Option<String>,
    
    /// Network name
    pub network: String,
    
    /// Timeout in seconds
    pub timeout: u64,
    
    /// Max retries
    pub max_retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rpc_endpoint: "http://localhost:8545".to_string(),
            contract_address: "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9".to_string(),
            prover_binary: "cargo".to_string(),
            circuit_path: "./circuits".to_string(),
            policy_root: "0x2428aab3614b2c3fd9683eb5b71378c0680a09fc693ba58129ae8bddd8bb534e".to_string(),
            genesis_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            private_key: None,
            network: "localhost".to_string(),
            timeout: 30,
            max_retries: 3,
        }
    }
}

impl Config {
    /// Load from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::ConfigError(format!("Failed to read config: {}", e)))?;
        
        let config = serde_json::from_str(&content)
            .map_err(|e| Error::ConfigError(format!("Failed to parse config: {}", e)))?;
        
        Ok(config)
    }
    
    /// Load from environment
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        
        Ok(Config {
            rpc_endpoint: std::env::var("RPC_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8545".to_string()),
            contract_address: std::env::var("LINEAGE_VERIFIER")
                .unwrap_or_else(|_| "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9".to_string()),
            prover_binary: std::env::var("PROVER_BINARY")
                .unwrap_or_else(|_| "cargo".to_string()),
            circuit_path: std::env::var("CIRCUIT_PATH")
                .unwrap_or_else(|_| "./circuits".to_string()),
            policy_root: std::env::var("POLICY_ROOT")
                .unwrap_or_else(|_| "0x2428aab3614b2c3fd9683eb5b71378c0680a09fc693ba58129ae8bddd8bb534e".to_string()),
            genesis_hash: std::env::var("GENESIS_HASH")
                .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()),
            private_key: std::env::var("PRIVATE_KEY").ok(),
            network: std::env::var("NETWORK")
                .unwrap_or_else(|_| "localhost".to_string()),
            timeout: std::env::var("TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            max_retries: std::env::var("MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
        })
    }
    
    /// Save to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| Error::ConfigError(format!("Failed to serialize: {}", e)))?;
        
        fs::write(path, json)
            .map_err(|e| Error::ConfigError(format!("Failed to write: {}", e)))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.network, "localhost");
    }
}