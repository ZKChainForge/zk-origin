// core/src/origin/policy.rs

use crate::origin::detector::OriginClass;
use serde::{Deserialize, Serialize};

/// Origin policy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OriginPolicy {
    /// Allowed transitions: from → to
    pub allowed: [[bool; 7]; 7],
    
    /// Rate limits per origin class
    pub rate_limits: [u32; 7],
    
    /// Epoch duration in seconds
    pub epoch_duration: u64,
}

impl OriginPolicy {
    /// Create default policy
    pub fn default_policy() -> Self {
        let mut policy = OriginPolicy {
            allowed: [[false; 7]; 7],
            rate_limits: [1, u32::MAX, 10, 100, 5, 1000, 1],
            epoch_duration: 86400,
        };
        
        // Genesis → User, Admin, System
        policy.allowed[OriginClass::Genesis as usize][OriginClass::User as usize] = true;
        policy.allowed[OriginClass::Genesis as usize][OriginClass::Admin as usize] = true;
        policy.allowed[OriginClass::Genesis as usize][OriginClass::System as usize] = true;
        
        // User → User
        policy.allowed[OriginClass::User as usize][OriginClass::User as usize] = true;
        
        // Admin → User, Admin, Bridge, System
        policy.allowed[OriginClass::Admin as usize][OriginClass::User as usize] = true;
        policy.allowed[OriginClass::Admin as usize][OriginClass::Admin as usize] = true;
        policy.allowed[OriginClass::Admin as usize][OriginClass::Bridge as usize] = true;
        policy.allowed[OriginClass::Admin as usize][OriginClass::System as usize] = true;
        
        // Bridge → User
        policy.allowed[OriginClass::Bridge as usize][OriginClass::User as usize] = true;
        
        // Governance → All
        for to in 0..7 {
            policy.allowed[OriginClass::Governance as usize][to] = true;
        }
        
        // System → User, System
        policy.allowed[OriginClass::System as usize][OriginClass::User as usize] = true;
        policy.allowed[OriginClass::System as usize][OriginClass::System as usize] = true;
        
        // Emergency → User, Admin, System
        policy.allowed[OriginClass::Emergency as usize][OriginClass::User as usize] = true;
        policy.allowed[OriginClass::Emergency as usize][OriginClass::Admin as usize] = true;
        policy.allowed[OriginClass::Emergency as usize][OriginClass::System as usize] = true;
        
        policy
    }
    
    /// Check if transition is allowed
    pub fn is_allowed(&self, from: OriginClass, to: OriginClass) -> bool {
        self.allowed[from as usize][to as usize]
    }
    
    /// Get rate limit
    pub fn get_rate_limit(&self, origin: OriginClass) -> u32 {
        self.rate_limits[origin as usize]
    }
}

impl Default for OriginPolicy {
    fn default() -> Self {
        Self::default_policy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_policy() {
        let policy = OriginPolicy::default();
        assert!(policy.is_allowed(OriginClass::User, OriginClass::User));
        assert!(!policy.is_allowed(OriginClass::User, OriginClass::Admin));
    }
}