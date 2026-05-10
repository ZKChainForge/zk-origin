use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Nova IVC configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NovaConfig {
    /// Compression threshold (max proof size before compression)
    pub compression_threshold: usize,

    /// Enable Groth16 compression
    pub groth16_compression: bool,

    /// Maximum state size in bytes
    pub max_state_size: usize,

    /// Maximum steps per proof
    pub max_steps: usize,

    /// Circuit version
    pub circuit_version: u32,

    /// Enable proof caching
    pub enable_caching: bool,

    /// Hash algorithm (0 = SHA3, 1 = BLAKE3)
    pub hash_algorithm: u8,

    /// Proof timeout in seconds
    pub proof_timeout_secs: u64,
}

impl NovaConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_state_size == 0 {
            return Err(crate::error::NovaError::SetupFailed {
                context: "max_state_size must be > 0".to_string(),
            });
        }

        if self.max_steps == 0 {
            return Err(crate::error::NovaError::SetupFailed {
                context: "max_steps must be > 0".to_string(),
            });
        }

        if self.compression_threshold == 0 {
            return Err(crate::error::NovaError::SetupFailed {
                context: "compression_threshold must be > 0".to_string(),
            });
        }

        if self.hash_algorithm > 1 {
            return Err(crate::error::NovaError::SetupFailed {
                context: "Invalid hash_algorithm".to_string(),
            });
        }

        Ok(())
    }

    /// Production configuration
    pub fn production() -> Self {
        NovaConfig {
            compression_threshold: 2500,
            groth16_compression: true,
            max_state_size: 48,
            max_steps: 1_000_000,
            circuit_version: 1,
            enable_caching: true,
            hash_algorithm: 0,       // SHA3
            proof_timeout_secs: 300, // 5 minutes
        }
    }

    /// Development configuration
    pub fn development() -> Self {
        NovaConfig {
            compression_threshold: 10000,
            groth16_compression: false,
            max_state_size: 256,
            max_steps: 10000,
            circuit_version: 1,
            enable_caching: false,
            hash_algorithm: 0,
            proof_timeout_secs: 3600,
        }
    }

    /// Testing configuration
    pub fn testing() -> Self {
        NovaConfig {
            compression_threshold: 50000,
            groth16_compression: false,
            max_state_size: 256,
            max_steps: 1000,
            circuit_version: 1,
            enable_caching: false,
            hash_algorithm: 0,
            proof_timeout_secs: 10,
        }
    }
}

impl Default for NovaConfig {
    fn default() -> Self {
        Self::production()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = NovaConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = NovaConfig::default();
        config.max_steps = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_production_config() {
        let config = NovaConfig::production();
        assert!(config.validate().is_ok());
        assert!(config.groth16_compression);
    }
}
