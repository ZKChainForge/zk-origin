//! State management module
//!
//! Provides state representation and state machine implementation

/// State hashing utilities
pub mod hash;
/// State machine implementation
pub mod machine;

pub use hash::{hash_state, hash_transition, keccak256};
pub use machine::{AccountState, Lineage, State, StateData, StateMachine};
