//! Core type definitions for ZK-ORIGIN

pub mod origin;
pub mod lineage;
pub mod transition;
pub mod policy;
pub mod witness;
pub mod proof;
pub mod error;

pub use origin::OriginClass;
pub use lineage::LineageCommitment;
pub use transition::Transition;
pub use policy::OriginPolicy;
pub use witness::StepWitness;
pub use proof::LineageProof;
pub use error::{ZkOriginError, Result};