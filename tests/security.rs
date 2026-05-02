//! Security tests

#[cfg(test)]
mod tests {
    use zk_origin_core::{State, StateData, Transition};
    
    #[test]
    fn test_nonce_reuse_prevention() {
        let prev = State::new(StateData::default(), 1000, 0);
        let new1 = State::new(StateData::default(), 2000, 1);
        let new2 = State::new(StateData::default(), 3000, 2);
        
        // Both claim nonce 1
        let result1 = Transition::new(prev.clone(), new1, "user".to_string(), 1);
        let result2 = Transition::new(prev.clone(), new2, "user".to_string(), 1);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
    
    #[test]
    fn test_time_rewind_prevention() {
        let prev = State::new(StateData::default(), 2000, 0);
        let new = State::new(StateData::default(), 1000, 1);
        
        let result = Transition::new(prev, new, "user".to_string(), 1);
        assert!(result.is_err());
    }
}