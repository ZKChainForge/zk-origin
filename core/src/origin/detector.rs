use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

/// Origin class with formal guarantees
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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
    /// Convert to u8
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Convert from u8 with validation
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

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            OriginClass::Genesis => "Genesis",
            OriginClass::User => "User",
            OriginClass::Admin => "Admin",
            OriginClass::Bridge => "Bridge",
            OriginClass::Governance => "Governance",
            OriginClass::System => "System",
            OriginClass::Emergency => "Emergency",
        }
    }

    /// Get all classes
    pub fn all() -> [OriginClass; 7] {
        [
            OriginClass::Genesis,
            OriginClass::User,
            OriginClass::Admin,
            OriginClass::Bridge,
            OriginClass::Governance,
            OriginClass::System,
            OriginClass::Emergency,
        ]
    }
}

impl FromStr for OriginClass {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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
        write!(f, "{}", self.name())
    }
}

/// Origin detection context with validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OriginContext {
    /// Address of the initiator
    pub initiator_address: String,

    /// Optional key of the initiator
    pub initiator_key: Option<String>,

    /// Optional source chain for bridge origins
    pub source_chain: Option<String>,

    /// Optional proposal ID for governance
    pub proposal_id: Option<u64>,

    /// Timestamp when origin was created
    pub timestamp: u64,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl OriginContext {
    /// Validate context consistency
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.initiator_address.is_empty() {
            return Err(crate::error::Error::invalid_origin_class(
                "initiator_address cannot be empty",
            ));
        }

        Ok(())
    }
}

/// Production-grade origin detector
pub struct OriginDetector {
    genesis_address: String,
    admin_addresses: HashSet<String>,
    bridge_addresses: HashSet<String>,
    governance_address: String,
    system_address: String,
    emergency_key: String,
}

impl OriginDetector {
    /// Create new detector with validation
    pub fn new(
        genesis_address: String,
        admin_addresses: Vec<String>,
        bridge_addresses: Vec<String>,
        governance_address: String,
        system_address: String,
        emergency_key: String,
    ) -> crate::error::Result<Self> {
        // Validate no empty addresses
        if genesis_address.is_empty()
            || governance_address.is_empty()
            || system_address.is_empty()
            || emergency_key.is_empty()
        {
            return Err(crate::error::Error::invalid_origin_class(
                "Required addresses cannot be empty",
            ));
        }

        Ok(OriginDetector {
            genesis_address,
            admin_addresses: admin_addresses.into_iter().collect(),
            bridge_addresses: bridge_addresses.into_iter().collect(),
            governance_address,
            system_address,
            emergency_key,
        })
    }

    /// Detect origin class from context
    pub fn detect(&self, context: &OriginContext) -> crate::error::Result<OriginClass> {
        context.validate()?;

        // Check in priority order
        if context.initiator_address == self.genesis_address {
            return Ok(OriginClass::Genesis);
        }

        if self.admin_addresses.contains(&context.initiator_address) {
            return Ok(OriginClass::Admin);
        }

        if self.bridge_addresses.contains(&context.initiator_address) {
            return Ok(OriginClass::Bridge);
        }

        if context.initiator_address == self.governance_address {
            return Ok(OriginClass::Governance);
        }

        if context.initiator_address == self.system_address {
            return Ok(OriginClass::System);
        }

        if context.initiator_key.as_ref() == Some(&self.emergency_key) {
            return Ok(OriginClass::Emergency);
        }

        // Default to User
        Ok(OriginClass::User)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_conversion() {
        for class in OriginClass::all() {
            assert_eq!(OriginClass::from_u8(class.as_u8()), Some(class));
        }
    }

    #[test]
    fn test_origin_parsing() {
        assert_eq!("user".parse::<OriginClass>().unwrap(), OriginClass::User);
        assert_eq!("admin".parse::<OriginClass>().unwrap(), OriginClass::Admin);
        assert!("invalid".parse::<OriginClass>().is_err());
    }

    #[test]
    fn test_detector_creation() {
        let detector = OriginDetector::new(
            "genesis".to_string(),
            vec!["admin1".to_string()],
            vec!["bridge".to_string()],
            "governance".to_string(),
            "system".to_string(),
            "emergency_key".to_string(),
        );
        assert!(detector.is_ok());
    }

    #[test]
    fn test_detector_detection() {
        let detector = OriginDetector::new(
            "genesis".to_string(),
            vec!["admin1".to_string()],
            vec![],
            "governance".to_string(),
            "system".to_string(),
            "emergency_key".to_string(),
        )
        .unwrap();

        let ctx = OriginContext {
            initiator_address: "admin1".to_string(),
            initiator_key: None,
            source_chain: None,
            proposal_id: None,
            timestamp: 0,
            metadata: std::collections::HashMap::new(),
        };

        assert_eq!(detector.detect(&ctx).unwrap(), OriginClass::Admin);
    }
}
