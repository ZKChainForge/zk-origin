use super::Origin;

/// Origin transition policy
#[derive(Clone, Debug)]
pub struct Policy {
    /// Allowed transitions as (from, to) pairs
    allowed: Vec<(Origin, Origin)>,
}

impl Policy {
    /// Create the default ZK-ORIGIN policy
    pub fn default_policy() -> Self {
        Self {
            allowed: vec![
                (Origin::Genesis, Origin::User),
                (Origin::Genesis, Origin::Admin),
                (Origin::User, Origin::User),
                (Origin::Admin, Origin::User),
                (Origin::Admin, Origin::Admin),
                (Origin::Admin, Origin::Bridge),
                (Origin::Bridge, Origin::User),
            ],
        }
    }
    
    /// Check if a transition is allowed
    pub fn is_allowed(&self, from: Origin, to: Origin, depth: u64) -> bool {
        // Never allow returning to Genesis after depth 0
        if depth > 0 && to == Origin::Genesis {
            return false;
        }
        
        self.allowed.contains(&(from, to))
    }
    
    /// Get the Merkle root of the policy
    pub fn merkle_root(&self) -> [u8; 32] {
        // In production, compute actual Merkle root
        [0u8; 32] // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_policy() {
        let policy = Policy::default_policy();
        
        // Allowed
        assert!(policy.is_allowed(Origin::Genesis, Origin::User, 0));
        assert!(policy.is_allowed(Origin::User, Origin::User, 1));
        assert!(policy.is_allowed(Origin::Admin, Origin::Bridge, 1));
        
        // Not allowed
        assert!(!policy.is_allowed(Origin::User, Origin::Admin, 1));
        assert!(!policy.is_allowed(Origin::Bridge, Origin::Admin, 1));
        assert!(!policy.is_allowed(Origin::User, Origin::Genesis, 5));
    }
}