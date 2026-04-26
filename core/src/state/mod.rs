// core/src/state/mod.rs

pub mod machine;
pub mod hash;

pub use machine::{State, StateData, AccountState, StateMachine, Lineage};
pub use hash::{keccak256, hash_state, hash_transition};