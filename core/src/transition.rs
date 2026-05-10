use crate::{
    error::{Error, Result},
    OriginClass, State,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Production state transition with full validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    pub prev_state: State,
    pub new_state: State,
    pub prev_origin: OriginClass,
    pub new_origin: OriginClass,
    pub timestamp: u64,
    pub nonce: u64,
    pub initiator: String,
    pub metadata: HashMap<String, String>,
}

impl Transition {
    /// Create new transition with complete validation
    pub fn new(
        prev_state: State,
        new_state: State,
        prev_origin: OriginClass,
        new_origin: OriginClass,
        initiator: String,
        nonce: u64,
    ) -> Result<Self> {
        // Validate states
        prev_state.validate()?;
        new_state.validate()?;

        // Validate nonce increases
        if nonce <= prev_state.nonce {
            return Err(Error::invalid_nonce(format!(
                "Nonce {} must be > {}",
                nonce, prev_state.nonce
            )));
        }

        if nonce == u64::MAX {
            return Err(Error::invalid_nonce("Nonce overflow"));
        }

        // States must be different
        if prev_state.hash == new_state.hash {
            return Err(Error::StateDifferenceFailed);
        }

        // Time must move forward
        if new_state.timestamp < prev_state.timestamp {
            return Err(Error::invalid_timestamp(format!(
                "Time regression: {} < {}",
                new_state.timestamp, prev_state.timestamp
            )));
        }

        // Initiator cannot be empty
        if initiator.is_empty() {
            return Err(Error::authorization_failed("Initiator cannot be empty"));
        }

        Ok(Transition {
            prev_state,
            new_state,
            prev_origin,
            new_origin,
            timestamp: new_state.timestamp,
            nonce,
            initiator,
            metadata: HashMap::new(),
        })
    }

    /// Validate transition completely
    pub fn validate(&self) -> Result<()> {
        // Validate states
        self.prev_state.validate()?;
        self.new_state.validate()?;

        // Check nonce
        if self.nonce <= self.prev_state.nonce {
            return Err(Error::invalid_nonce("Nonce not increasing"));
        }

        // Check states different
        if self.prev_state.hash == self.new_state.hash {
            return Err(Error::StateDifferenceFailed);
        }

        // Check time
        if self.new_state.timestamp < self.prev_state.timestamp {
            return Err(Error::invalid_timestamp("Time not moving forward"));
        }

        // Check initiator
        if self.initiator.is_empty() {
            return Err(Error::authorization_failed("Initiator cannot be empty"));
        }

        Ok(())
    }

    /// Get transition hash
    pub fn hash(&self) -> crate::hash::Hash {
        crate::hash::hash_transition(
            self.prev_state.hash,
            self.new_state.hash,
            self.new_origin.as_u8(),
            self.timestamp,
            self.nonce,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StateData;

    #[test]
    fn test_transition_creation() {
        let prev = State::new(StateData::default(), 1000, 0).unwrap();
        let new = State::new(StateData::default(), 2000, 1).unwrap();

        let result = Transition::new(
            prev,
            new,
            OriginClass::User,
            OriginClass::User,
            "user".to_string(),
            1,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_nonce_must_increase() {
        let prev = State::new(StateData::default(), 1000, 5).unwrap();
        let new = State::new(StateData::default(), 2000, 6).unwrap();

        let result = Transition::new(
            prev,
            new,
            OriginClass::User,
            OriginClass::User,
            "user".to_string(),
            4, // Less than prev nonce
        );

        assert!(result.is_err());
    }
}
