use serde::{Deserialize, Serialize};

/// Origin class for state transitions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Origin {
    /// Genesis state (initial deployment)
    Genesis = 0,
    /// Normal user transaction
    User = 1,
    /// Administrative action
    Admin = 2,
    /// Cross-chain bridge import
    Bridge = 3,
}

impl Origin {
    /// Get the numeric value of the origin
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    
    /// Try to create from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Origin::Genesis),
            1 => Some(Origin::User),
            2 => Some(Origin::Admin),
            3 => Some(Origin::Bridge),
            _ => None,
        }
    }
    
    /// Get the name of the origin
    pub fn name(&self) -> &'static str {
        match self {
            Origin::Genesis => "Genesis",
            Origin::User => "User",
            Origin::Admin => "Admin",
            Origin::Bridge => "Bridge",
        }
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_origin_round_trip() {
        for i in 0..4 {
            let origin = Origin::from_u8(i).unwrap();
            assert_eq!(origin.as_u8(), i);
        }
    }
    
    #[test]
    fn test_invalid_origin() {
        assert!(Origin::from_u8(5).is_none());
    }
}