//! Origin class definitions

use serde::{Deserialize, Serialize};
use std::fmt;

/// Origin classes represent the type of entity that authorized a state transition.
///
/// Each transition in a state's lineage is tagged with an origin class,
/// allowing the protocol to enforce policies about which transitions are allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OriginClass {
    /// Genesis state - the initial state of the system
    Genesis = 0,
    
    /// User transaction - normal user-initiated state change
    User = 1,
    
    /// Admin operation - privileged administrative action
    Admin = 2,
    
    /// Bridge import - state imported from another chain
    Bridge = 3,
    
    /// Governance action - DAO-approved state change
    Governance = 4,
    
    /// System operation - automated protocol-level change
    System = 5,
}

impl OriginClass {
    /// Returns all origin classes
    pub fn all() -> &'static [OriginClass] {
        &[
            OriginClass::Genesis,
            OriginClass::User,
            OriginClass::Admin,
            OriginClass::Bridge,
            OriginClass::Governance,
            OriginClass::System,
        ]
    }

    /// Convert to field element representation
    pub fn to_field_element(&self) -> u64 {
        *self as u64
    }

    /// Try to convert from u8
    pub fn try_from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(OriginClass::Genesis),
            1 => Some(OriginClass::User),
            2 => Some(OriginClass::Admin),
            3 => Some(OriginClass::Bridge),
            4 => Some(OriginClass::Governance),
            5 => Some(OriginClass::System),
            _ => None,
        }
    }

    /// Get the default rate limit for this origin class per epoch
    pub fn default_rate_limit(&self) -> u32 {
        match self {
            OriginClass::Genesis => 1,
            OriginClass::User => u32::MAX,
            OriginClass::Admin => 10,
            OriginClass::Bridge => 100,
            OriginClass::Governance => 5,
            OriginClass::System => 1000,
        }
    }

    /// Returns true if this is a privileged origin class
    pub fn is_privileged(&self) -> bool {
        matches!(
            self,
            OriginClass::Admin | OriginClass::Governance | OriginClass::System
        )
    }
}

impl fmt::Display for OriginClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OriginClass::Genesis => write!(f, "Genesis"),
            OriginClass::User => write!(f, "User"),
            OriginClass::Admin => write!(f, "Admin"),
            OriginClass::Bridge => write!(f, "Bridge"),
            OriginClass::Governance => write!(f, "Governance"),
            OriginClass::System => write!(f, "System"),
        }
    }
}

impl Default for OriginClass {
    fn default() -> Self {
        OriginClass::User
    }
}

impl TryFrom<u8> for OriginClass {
    type Error = &'static str;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        OriginClass::try_from_u8(value).ok_or("Invalid origin class value")
    }
}

impl From<OriginClass> for u8 {
    fn from(origin: OriginClass) -> Self {
        origin as u8
    }
}

impl From<OriginClass> for u64 {
    fn from(origin: OriginClass) -> Self {
        origin as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_class_values() {
        assert_eq!(OriginClass::Genesis as u8, 0);
        assert_eq!(OriginClass::User as u8, 1);
        assert_eq!(OriginClass::Admin as u8, 2);
        assert_eq!(OriginClass::Bridge as u8, 3);
        assert_eq!(OriginClass::Governance as u8, 4);
        assert_eq!(OriginClass::System as u8, 5);
    }

    #[test]
    fn test_origin_class_conversion() {
        for class in OriginClass::all() {
            let value = *class as u8;
            let recovered = OriginClass::try_from_u8(value).unwrap();
            assert_eq!(*class, recovered);
        }
    }

    #[test]
    fn test_invalid_origin_class() {
        assert!(OriginClass::try_from_u8(6).is_none());
        assert!(OriginClass::try_from_u8(255).is_none());
    }

    #[test]
    fn test_rate_limits() {
        assert_eq!(OriginClass::Genesis.default_rate_limit(), 1);
        assert_eq!(OriginClass::User.default_rate_limit(), u32::MAX);
        assert_eq!(OriginClass::Admin.default_rate_limit(), 10);
    }

    #[test]
    fn test_privileged() {
        assert!(!OriginClass::User.is_privileged());
        assert!(OriginClass::Admin.is_privileged());
        assert!(OriginClass::Governance.is_privileged());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", OriginClass::User), "User");
        assert_eq!(format!("{}", OriginClass::Admin), "Admin");
    }

    #[test]
    fn test_serialization() {
        let origin = OriginClass::Admin;
        let json = serde_json::to_string(&origin).unwrap();
        let recovered: OriginClass = serde_json::from_str(&json).unwrap();
        assert_eq!(origin, recovered);
    }
}