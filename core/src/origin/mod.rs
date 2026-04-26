// core/src/origin/mod.rs

pub mod detector;
pub mod auth;
pub mod policy;

pub use detector::{OriginClass, OriginDetector, OriginContext};
pub use auth::AuthorizationVerifier;
pub use policy::OriginPolicy;