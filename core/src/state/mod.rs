//! State management module
//!
//! Provides state representation and state machine implementation

/// State machine implementation
pub mod machine;
/// State hashing utilities
pub mod hash;

pub use machine::{State, StateData, AccountState, StateMachine, Lineage};
pub use hash::{keccak256, hash_state, hash_transition};