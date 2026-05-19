//! Witness management

/// Witness generation
pub mod generator;
/// Witness serialization
pub mod serializer;
/// Witness validation
pub mod validator;

pub use generator::{PrivateWitness, PublicWitness, TransitionWitness, WitnessGenerator};
pub use serializer::WitnessSerializer;
pub use validator::WitnessValidator;