use zk_origin_prover::{LineageProver, Origin, Transition};

#[test]
fn test_user_chain() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    // Genesis -> User -> User -> User
    let mut prev_state = genesis_state;
    let origins = [Origin::User, Origin::User, Origin::User];
    
    for (i, origin) in origins.iter().enumerate() {
        let new_state = [(i + 1) as u8; 32];
        prover.add_transition(Transition::new(
            prev_state,
            new_state,
            *origin,
            (i + 1) as u64 * 1000,
        )).unwrap();
        prev_state = new_state;
    }
    
    assert_eq!(prover.current_lineage().depth, 3);
}

#[test]
fn test_admin_to_bridge_chain() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    // Genesis -> Admin -> Admin -> Bridge -> User
    let transitions = [
        Origin::Admin,
        Origin::Admin,
        Origin::Bridge,
        Origin::User,
    ];
    
    let mut prev_state = genesis_state;
    
    for (i, origin) in transitions.iter().enumerate() {
        let new_state = [(i + 1) as u8; 32];
        prover.add_transition(Transition::new(
            prev_state,
            new_state,
            *origin,
            (i + 1) as u64 * 1000,
        )).unwrap();
        prev_state = new_state;
    }
    
    assert_eq!(prover.current_lineage().depth, 4);
}

#[test]
fn test_long_chain() {
    let genesis_state = [0u8; 32];
    let mut prover = LineageProver::new(genesis_state).unwrap();
    
    let mut prev_state = genesis_state;
    
    // 100 user transactions
    for i in 0..100 {
        let new_state = [(i + 1) as u8; 32];
        prover.add_transition(Transition::new(
            prev_state,
            new_state,
            Origin::User,
            (i + 1) as u64 * 1000,
        )).unwrap();
        prev_state = new_state;
    }
    
    assert_eq!(prover.current_lineage().depth, 100);
    
    // Finalize should work
    let proof = prover.finalize().unwrap();
    assert!(proof.verify());
}