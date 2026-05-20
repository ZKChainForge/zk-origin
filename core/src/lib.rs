#![warn(missing_docs)]

//! Provides cryptographic state lineage verification with origin-based authorization

/// Error handling module
pub mod error;
/// Cryptographic hashing utilities
pub mod hash;
/// Origin detection and authorization
pub mod origin;
/// Origin-based policy enforcement
pub mod policy;
/// State management
pub mod state;
/// State transition logic
pub mod transition;
/// Utility functions
pub mod utils;

pub use error::{Error, Result};
pub use hash::{hash_lineage, hash_state, hash_transition, keccak256, Hash};
pub use origin::auth::AuthorizationVerifier;
pub use origin::{OriginClass, OriginContext, OriginDetector};
pub use policy::OriginPolicy;
pub use state::{Lineage, State, StateData, StateMachine};
pub use transition::Transition;

/// Current version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Production constants
pub mod consts {
    /// Maximum lineage depth
    pub const MAX_LINEAGE_DEPTH: u32 = 1_000_000;

    /// Epoch duration in seconds (24 hours)
    pub const EPOCH_DURATION_SECS: u64 = 86400;

    /// Number of origin classes
    pub const NUM_ORIGIN_CLASSES: usize = 7;
}