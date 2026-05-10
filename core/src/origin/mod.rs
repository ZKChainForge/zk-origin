//! Origin detection and authorization module
//!
//! Detects which origin class initiated a state transition and verifies authorization

pub mod auth;
pub mod detector;
/// Origin policy module
pub mod policy;

pub use auth::AuthorizationVerifier;
pub use detector::{OriginClass, OriginContext, OriginDetector};
pub use policy::OriginPolicy;
