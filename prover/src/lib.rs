//! ZK-ORIGIN Recursive Prover
//! 
//! This crate provides Nova-based recursive proof generation for state lineage verification.
//! 
//! # Overview
//! 
//! The prover generates constant-size proofs that attest to:
//! 1. Valid origin transitions according to policy
//! 2. Correct lineage commitment updates
//! 3. Unbroken chain from genesis
//! 
//! # Example
//! 
//! ```rust,ignore
//! use zk_origin_prover::{LineageProver, Origin, Transition};
//! 
//! let mut prover = LineageProver::new()?;
//! 
//! // Add transitions
//! prover.add_transition(Transition {
//!     prev_state: prev_hash,
//!     new_state: new_hash,
//!     origin: Origin::User,
//!     timestamp: now,
//! })?;
//! 
//! // Generate final proof
//! let proof = prover.finalize()?;
//! ```

pub mod types;
pub mod circuit;
pub mod prover;
pub mod utils;

pub use types::{Origin, Transition, LineageCommitment};
pub use prover::LineageProver;