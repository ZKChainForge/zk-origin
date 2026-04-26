//! Origin Class Detection
//!
//! Detects which origin class initiated a state transition:
//! - User: Normal user transaction
//! - Admin: Multisig approval
//! - Bridge: Cross-chain import
//! - Governance: Proposal execution
//! - System: System call
//! - Emergency: Emergency intervention

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Origin class enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OriginClass {
    Genesis = 0,
    User = 1,
    Admin = 2,
    Bridge = 3,
    Governance = 4,
    System = 5,
    Emergency = 6,
}

impl OriginClass {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(OriginClass::Genesis),
            1 => Some(OriginClass::User),
            2 => Some(OriginClass::Admin),
            3 => Some(OriginClass::Bridge),
            4 => Some(OriginClass::Governance),
            5 => Some(OriginClass::System),
            6 => Some(OriginClass::Emergency),
            _ => None,
        }
    }
}

impl FromStr for OriginClass {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "genesis" => Ok(OriginClass::Genesis),
            "user" => Ok(OriginClass::User),
            "admin" => Ok(OriginClass::Admin),
            "bridge" => Ok(OriginClass::Bridge),
            "governance" => Ok(OriginClass::Governance),
            "system" => Ok(OriginClass::System),
            "emergency" => Ok(OriginClass::Emergency),
            _ => Err(format!("Unknown origin class: {}", s)),
        }
    }
}

impl std::fmt::Display for OriginClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OriginClass::Genesis => write!(f, "Genesis"),
            OriginClass::User => write!(f, "User"),
            OriginClass::Admin => write!(f, "Admin"),
            OriginClass::Bridge => write!(f, "Bridge"),
            OriginClass::Governance => write!(f, "Governance"),
            OriginClass::System => write!(f, "System"),
            OriginClass::Emergency => write!(f, "Emergency"),
        }
    }
}

/// Origin detection context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OriginContext {
    pub initiator_address: String,
    pub initiator_key: Option<String>,
    pub source_chain: Option<String>,
    pub proposal_id: Option<u64>,
    pub timestamp: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Origin detector
pub struct OriginDetector {
    genesis_address: String,
    admin_addresses: Vec<String>,
    bridge_addresses: Vec<String>,
    governance_address: String,
    system_address: String,
    emergency_key: String,
}

impl OriginDetector {
    pub fn new(
        genesis_address: String,
        admin_addresses: Vec<String>,
        bridge_addresses: Vec<String>,
        governance_address: String,
        system_address: String,
        emergency_key: String,
    ) -> Self {
        OriginDetector {
            genesis_address,
            admin_addresses,
            bridge_addresses,
            governance_address,
            system_address,
            emergency_key,
        }
    }
    
    /// Detect origin class from context
    pub fn detect(&self, context: &OriginContext) -> OriginClass {
        // Check in priority order
        if context.initiator_address == self.genesis_address {
            return OriginClass::Genesis;
        }
        
        if self.admin_addresses.contains(&context.initiator_address) {
            return OriginClass::Admin;
        }
        
        if self.bridge_addresses.contains(&context.initiator_address) {
            return OriginClass::Bridge;
        }
        
        if context.initiator_address == self.governance_address {
            return OriginClass::Governance;
        }
        
        if context.initiator_address == self.system_address {
            return OriginClass::System;
        }
        
        if context.initiator_key.as_ref() == Some(&self.emergency_key) {
            return OriginClass::Emergency;
        }
        
        // Default to User
        OriginClass::User
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_origin_from_u8() {
        assert_eq!(OriginClass::from_u8(1), Some(OriginClass::User));
        assert_eq!(OriginClass::from_u8(2), Some(OriginClass::Admin));
        assert_eq!(OriginClass::from_u8(99), None);
    }
    
    #[test]
    fn test_origin_detection() {
        let detector = OriginDetector::new(
            "genesis".to_string(),
            vec!["admin1".to_string()],
            vec!["bridge".to_string()],
            "governance".to_string(),
            "system".to_string(),
            "emergency_key".to_string(),
        );
        
        let ctx = OriginContext {
            initiator_address: "admin1".to_string(),
            initiator_key: None,
            source_chain: None,
            proposal_id: None,
            timestamp: 0,
            metadata: std::collections::HashMap::new(),
        };
        
        assert_eq!(detector.detect(&ctx), OriginClass::Admin);
    }
}