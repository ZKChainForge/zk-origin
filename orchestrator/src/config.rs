//! Configuration

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::error::{Error, Result};

/// Orchestrator configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// RPC endpoint
    pub rpc_endpoint: String,
    
    /// Contract address
    pub contract_address: String,
    
    /// Circuit path
    pub circuit_path: String,
    
    /// Policy root
    pub policy_root: String,
    
    /// Genesis hash
    pub genesis_hash: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rpc_endpoint: "http://localhost:8545".to_string(),
            contract_address: "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9".to_string(),
            circuit_path: "./circuits".to_string(),
            policy_root: "0x2428aab3614b2c3fd9683eb5b71378c0680a09fc693ba58129ae8bddd8bb534e".to_string(),
            genesis_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        }
    }
}

impl Config {
    /// Load from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::ConfigError(format!("Failed to read config: {}", e)))?;
        
        let config = serde_json::from_str(&content)
            .map_err(|e| Error::ConfigError(format!("Failed to parse config: {}", e)))?;
        
        Ok(config)
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