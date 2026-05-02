//! Integration tests

#[cfg(test)]
mod tests {
    use zk_origin_core::{State, StateData, Transition, StateMachine, OriginPolicy};
    
    #[test]
    fn test_single_transition() {
        let genesis = State::genesis(StateData::default());
        let new_state = State::new(StateData::default(), 1000, 1);
        
        let transition = Transition::new(genesis.clone(), new_state, "user".to_string(), 1);
        assert!(transition.is_ok());
        
        let policy = OriginPolicy::default();
        let mut machine = StateMachine::new(genesis, policy);
        assert!(machine.apply_transition(transition.unwrap()).is_ok());
    }
    
    #[test]
    fn test_multi_step_chain() {
        let genesis = State::genesis(StateData::default());
        let policy = OriginPolicy::default();
        let mut machine = StateMachine::new(genesis.clone(), policy);
        
        for i in 1..=5 {
            let new_state = State::new(StateData::default(), 1000 + i as u64, i as u64);
            let transition = Transition::new(
                machine.get_current_state().clone(),
                new_state,
                "user".to_string(),
                i as u64,
            );
            
            assert!(transition.is_ok());
            assert!(machine.apply_transition(transition.unwrap()).is_ok());
        }
        
        let lineage = machine.get_lineage();
        assert_eq!(lineage.depth, 5);
    }
}