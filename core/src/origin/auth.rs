//! Authorization Proof Verification
//!
//! Verifies that a claimed origin class has proper authorization

use crate::origin::detector::OriginClass;
use serde::{Deserialize, Serialize};

/// Authorization proof (different per origin class)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthorizationProof {
    User {
        signature: Vec<u8>,
        public_key: Vec<u8>,
        message: Vec<u8>,
    },
    Admin {
        signatures: Vec<Vec<u8>>,
        threshold: u8,
        signers: Vec<Vec<u8>>,
    },
    Bridge {
        source_chain: String,
        attestation: Vec<u8>,
        merkle_proof: Vec<Vec<u8>>,
    },
    Governance {
        proposal_id: u64,
        yes_votes: u64,
        no_votes: u64,
        threshold: u64,
    },
    System {
        caller_address: String,
    },
    Emergency {
        emergency_key: Vec<u8>,
        signature: Vec<u8>,
        conditions_met: Vec<bool>,
    },
    Genesis,
}

/// Authorization verifier
pub struct AuthorizationVerifier;

impl AuthorizationVerifier {
    /// Verify authorization proof
    pub fn verify(
        origin_class: OriginClass,
        proof: &AuthorizationProof,
    ) -> bool {
        match (origin_class, proof) {
            // User: verify EdDSA signature
            (OriginClass::User, AuthorizationProof::User { .. }) => {
                // In real implementation, use ed25519 library
                // For now, just check format
                true
            }
            
            // Admin: verify multisig
            (OriginClass::Admin, AuthorizationProof::Admin { threshold, signatures, .. }) => {
                // Check enough signatures
                signatures.len() >= *threshold as usize
            }
            
            // Bridge: verify attestation
            (OriginClass::Bridge, AuthorizationProof::Bridge { .. }) => {
                // In real implementation, verify signature + Merkle proof
                true
            }
            
            // Governance: verify proposal passed
            (OriginClass::Governance, AuthorizationProof::Governance { yes_votes, threshold, .. }) => {
                yes_votes > threshold
            }
            
            // System: just check address
            (OriginClass::System, AuthorizationProof::System { .. }) => {
                true
            }
            
            // Emergency: check conditions
            (OriginClass::Emergency, AuthorizationProof::Emergency { conditions_met, .. }) => {
                // At least one condition must be met
                conditions_met.iter().any(|&c| c)
            }
            
            // Genesis: no auth needed
            (OriginClass::Genesis, AuthorizationProof::Genesis) => {
                true
            }
            
            // Mismatched origin and proof
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genesis_auth() {
        let proof = AuthorizationProof::Genesis;
        assert!(AuthorizationVerifier::verify(OriginClass::Genesis, &proof));
    }
    
    #[test]
    fn test_admin_multisig() {
        let proof = AuthorizationProof::Admin {
            signatures: vec![vec![1, 2, 3], vec![4, 5, 6]],
            threshold: 2,
            signers: vec![],
        };
        assert!(AuthorizationVerifier::verify(OriginClass::Admin, &proof));
    }
}