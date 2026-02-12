use zk_origin_prover::{LineageProver, Origin, Transition};

#[test]
fn test_single_step_genesis_to_user() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    assert_eq!(prover.current_lineage().depth, 0);
    
    prover.add_transition(Transition::new(
        genesis_state,
        [1u8; 32],
        Origin::User,
        1000,
    )).unwrap();
    
    assert_eq!(prover.current_lineage().depth, 1);
    assert_eq!(prover.num_transitions(), 1);
}

#[test]
fn test_single_step_genesis_to_admin() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    prover.add_transition(Transition::new(
        genesis_state,
        [1u8; 32],
        Origin::Admin,
        1000,
    )).unwrap();
    
    assert_eq!(prover.current_lineage().depth, 1);
}