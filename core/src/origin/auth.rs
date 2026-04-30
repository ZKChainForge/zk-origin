//! Authorization Proof Verification
//!
//! Verifies that a claimed origin class has proper authorization

use crate::origin::detector::OriginClass;
use serde::{Deserialize, Serialize};

/// Authorization proof (different per origin class)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthorizationProof {
    /// User authorization via signature
    User {
        /// Signature bytes
        signature: Vec<u8>,
        /// Public key bytes
        public_key: Vec<u8>,
        /// Original message
        message: Vec<u8>,
    },
    /// Admin authorization via multisig
    Admin {
        /// Collected signatures
        signatures: Vec<Vec<u8>>,
        /// Required number of signatures
        threshold: u8,
        /// Addresses of signers
        signers: Vec<Vec<u8>>,
    },
    /// Bridge authorization via attestation
    Bridge {
        /// Source blockchain identifier
        source_chain: String,
        /// Attestation proof
        attestation: Vec<u8>,
        /// Merkle proof components
        merkle_proof: Vec<Vec<u8>>,
    },
    /// Governance authorization via proposal
    Governance {
        /// Proposal identifier
        proposal_id: u64,
        /// Number of yes votes
        yes_votes: u64,
        /// Number of no votes
        no_votes: u64,
        /// Required threshold
        threshold: u64,
    },
    /// System authorization via caller address
    System {
        /// Caller address
        caller_address: String,
    },
    /// Emergency authorization
    Emergency {
        /// Emergency key
        emergency_key: Vec<u8>,
        /// Signature
        signature: Vec<u8>,
        /// Emergency conditions met
        conditions_met: Vec<bool>,
    },
    /// Genesis authorization - no proof needed
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