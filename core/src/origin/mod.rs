//! Origin detection and authorization module
//!
//! Detects which origin class initiated a state transition and verifies authorization

pub mod detector;
pub mod auth;
/// Origin policy module
pub mod policy;

pub use detector::{OriginClass, OriginDetector, OriginContext};
pub use auth::AuthorizationVerifier;
pub use policy::OriginPolicy;