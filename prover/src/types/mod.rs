//! Core type definitions for ZK-ORIGIN

pub mod error;
pub mod lineage;
pub mod origin;
pub mod policy;
pub mod proof;
pub mod transition;
pub mod witness;

pub use error::{Result, ZkOriginError};
pub use lineage::LineageCommitment;
pub use origin::OriginClass;
pub use policy::OriginPolicy;
pub use proof::LineageProof;
pub use transition::Transition;
pub use witness::StepWitness;
