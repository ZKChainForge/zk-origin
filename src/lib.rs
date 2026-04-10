//! ZK-ORIGIN: Zero-Knowledge State Provenance Protocol
//! 
//! This crate provides policy tree management and utilities for the ZK-ORIGIN protocol

pub mod policy;
pub mod types;
pub mod utils;
pub mod proof;

// Re-export commonly used types
pub use types::OriginClass;
pub use policy::{PolicyTree, PolicyProof};
pub use utils::*;
pub use proof::{Transition, LineageCommitment, Witness};