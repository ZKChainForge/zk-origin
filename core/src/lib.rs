#![warn(missing_docs)]

//! ZK-ORIGIN Core State Machine
//!
//! This module provides the core state machine implementation with origin-based
//! authorization and policy enforcement.

/// State management module
pub mod state;
/// Origin detection and authorization module
pub mod origin;
/// Re-export origin policy
pub mod policy;
/// Error types
pub mod error;
/// Hashing utilities
pub mod hash;
/// Utility functions
pub mod utils;
/// State transitions
pub mod transition;

pub use state::{State, StateData, StateMachine, Lineage};
pub use origin::{OriginClass, OriginDetector, OriginContext};
pub use origin::auth::AuthorizationVerifier;
pub use policy::OriginPolicy;
pub use error::{Error, Result};
pub use transition::Transition;

/// Current version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");