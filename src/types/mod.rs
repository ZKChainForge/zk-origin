

//! Common types used across ZK-ORIGIN

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Genesis),
            1 => Some(Self::User),
            2 => Some(Self::Admin),
            3 => Some(Self::Bridge),
            4 => Some(Self::Governance),
            5 => Some(Self::System),
            6 => Some(Self::Emergency),
            _ => None,
        }
    }
    
    pub fn all() -> Vec<Self> {
        vec![
            Self::Genesis,
            Self::User,
            Self::Admin,
            Self::Bridge,
            Self::Governance,
            Self::System,
            Self::Emergency,
        ]
    }
}

impl std::fmt::Display for OriginClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Genesis => write!(f, "Genesis"),
            Self::User => write!(f, "User"),
            Self::Admin => write!(f, "Admin"),
            Self::Bridge => write!(f, "Bridge"),
            Self::Governance => write!(f, "Governance"),
            Self::System => write!(f, "System"),
            Self::Emergency => write!(f, "Emergency"),
        }
    }
}