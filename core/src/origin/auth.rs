use crate::error::{Error, Result};
use crate::origin::detector::OriginClass;
use serde::{Deserialize, Serialize};

/// Authorization proof with validation
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

impl AuthorizationProof {
    /// Validate proof format
    pub fn validate(&self, origin_class: OriginClass) -> Result<()> {
        match (self, origin_class) {
            (
                AuthorizationProof::User {
                    signature,
                    public_key,
                    message,
                },
                OriginClass::User,
            ) => {
                if signature.is_empty() {
                    return Err(Error::invalid_origin_class(
                        "User signature cannot be empty",
                    ));
                }
                if public_key.is_empty() {
                    return Err(Error::invalid_origin_class(
                        "User public key cannot be empty",
                    ));
                }
                if message.is_empty() {
                    return Err(Error::invalid_origin_class("User message cannot be empty"));
                }
                Ok(())
            }

            (
                AuthorizationProof::Admin {
                    threshold,
                    signatures,
                    signers,
                },
                OriginClass::Admin,
            ) => {
                if *threshold == 0 {
                    return Err(Error::invalid_origin_class("Admin threshold must be > 0"));
                }
                if signatures.len() < *threshold as usize {
                    return Err(Error::authorization_failed(format!(
                        "Expected {} signatures, got {}",
                        threshold,
                        signatures.len()
                    )));
                }
                if signatures.len() != signers.len() {
                    return Err(Error::authorization_failed(
                        "Signatures and signers length mismatch",
                    ));
                }
                Ok(())
            }

            (
                AuthorizationProof::Bridge {
                    source_chain,
                    attestation,
                    ..
                },
                OriginClass::Bridge,
            ) => {
                if source_chain.is_empty() {
                    return Err(Error::invalid_origin_class(
                        "Bridge source_chain cannot be empty",
                    ));
                }
                if attestation.is_empty() {
                    return Err(Error::invalid_origin_class(
                        "Bridge attestation cannot be empty",
                    ));
                }
                Ok(())
            }

            (
                AuthorizationProof::Governance {
                    proposal_id,
                    yes_votes,
                    no_votes,
                    threshold,
                },
                OriginClass::Governance,
            ) => {
                if *threshold == 0 {
                    return Err(Error::invalid_origin_class(
                        "Governance threshold must be > 0",
                    ));
                }
                if yes_votes <= no_votes {
                    return Err(Error::authorization_failed(
                        "Governance yes votes must exceed no votes + threshold",
                    ));
                }
                if *yes_votes < threshold + no_votes {
                    return Err(Error::authorization_failed("Governance threshold not met"));
                }
                Ok(())
            }

            (AuthorizationProof::System { caller_address }, OriginClass::System) => {
                if caller_address.is_empty() {
                    return Err(Error::invalid_origin_class(
                        "System caller_address cannot be empty",
                    ));
                }
                Ok(())
            }

            (AuthorizationProof::Emergency { conditions_met, .. }, OriginClass::Emergency) => {
                if conditions_met.is_empty() {
                    return Err(Error::invalid_origin_class(
                        "Emergency must have conditions",
                    ));
                }
                if !conditions_met.iter().any(|&c| c) {
                    return Err(Error::authorization_failed(
                        "Emergency requires at least one condition to be met",
                    ));
                }
                Ok(())
            }

            (AuthorizationProof::Genesis, OriginClass::Genesis) => Ok(()),

            _ => Err(Error::authorization_failed(
                "Origin class and proof type mismatch",
            )),
        }
    }
}

/// Authorization verifier
pub struct AuthorizationVerifier;

impl AuthorizationVerifier {
    /// Verify authorization proof
    pub fn verify(origin_class: OriginClass, proof: &AuthorizationProof) -> Result<()> {
        proof.validate(origin_class)?;

        match (origin_class, proof) {
            (OriginClass::User, AuthorizationProof::User { .. }) => {
                // In production, verify Ed25519 signature
                Ok(())
            }

            (
                OriginClass::Admin,
                AuthorizationProof::Admin {
                    threshold,
                    signatures,
                    ..
                },
            ) => {
                if (signatures.len() as u8) < *threshold {
                    return Err(Error::authorization_failed("Insufficient signatures"));
                }
                Ok(())
            }

            (OriginClass::Bridge, AuthorizationProof::Bridge { .. }) => {
                // In production, verify Merkle proof and signature
                Ok(())
            }

            (
                OriginClass::Governance,
                AuthorizationProof::Governance {
                    yes_votes,
                    threshold,
                    ..
                },
            ) => {
                if yes_votes <= threshold {
                    return Err(Error::authorization_failed("Insufficient votes"));
                }
                Ok(())
            }

            (OriginClass::System, AuthorizationProof::System { .. }) => Ok(()),

            (OriginClass::Emergency, AuthorizationProof::Emergency { conditions_met, .. }) => {
                if !conditions_met.iter().any(|&c| c) {
                    return Err(Error::authorization_failed("No emergency conditions met"));
                }
                Ok(())
            }

            (OriginClass::Genesis, AuthorizationProof::Genesis) => Ok(()),

            _ => Err(Error::authorization_failed("Proof validation failed")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_proof_validation() {
        let proof = AuthorizationProof::Admin {
            signatures: vec![vec![1, 2, 3], vec![4, 5, 6]],
            threshold: 2,
            signers: vec![vec![1], vec![2]],
        };
        assert!(proof.validate(OriginClass::Admin).is_ok());
    }

    #[test]
    fn test_admin_insufficient_signatures() {
        let proof = AuthorizationProof::Admin {
            signatures: vec![vec![1, 2, 3]],
            threshold: 2,
            signers: vec![vec![1], vec![2]],
        };
        assert!(proof.validate(OriginClass::Admin).is_err());
    }
}
