use zk_origin::{
    policy::{PolicyTree, OriginClass, get_policy_leaves},
    prover::LineageProver,
};

#[test]
fn test_full_lineage_with_policy() {
    // 1. Build policy tree
    let transitions = get_policy_leaves();
    let policy_tree = PolicyTree::new(transitions);
    let policy_root = policy_tree.root();
    
    println!("Policy root: {:?}", policy_root);
    
    // 2. Initialize prover
    let mut prover = LineageProver::new_with_policy(policy_root);
    
    // 3. Genesis → User (allowed)
    let proof1 = policy_tree.prove(OriginClass::Genesis, OriginClass::User).unwrap();
    let result = prover.add_transition_with_policy(
        vec![1u8; 32],  // new state
        OriginClass::User,
        1000,  // timestamp
        proof1.path_elements.clone(),
        proof1.path_indices.clone(),
    );
    assert!(result.is_ok());
    
    // 4. User → User (allowed)
    let proof2 = policy_tree.prove(OriginClass::User, OriginClass::User).unwrap();
    let result = prover.add_transition_with_policy(
        vec![2u8; 32],
        OriginClass::User,
        2000,
        proof2.path_elements,
        proof2.path_indices,
    );
    assert!(result.is_ok());
    
    // 5. User → Admin (NOT allowed - should fail proof generation)
    let proof_invalid = policy_tree.prove(OriginClass::User, OriginClass::Admin);
    assert!(proof_invalid.is_none(), "User → Admin should not have proof");
}

#[test]
fn test_rate_limiting() {
    let policy_tree = PolicyTree::new(get_policy_leaves());
    let mut prover = LineageProver::new_with_policy(policy_tree.root());
    
    // Admin limit is 10 per epoch
    // Try to exceed it
    for i in 0..10 {
        let proof = policy_tree.prove(OriginClass::Genesis, OriginClass::Admin).unwrap();
        let result = prover.add_transition_with_policy(
            vec![i as u8; 32],
            OriginClass::Admin,
            1000 + i * 100,
            proof.path_elements.clone(),
            proof.path_indices.clone(),
        );
        assert!(result.is_ok(), "Transition {} should succeed", i);
    }
    
    // 11th should fail (rate limit exceeded)
    let proof = policy_tree.prove(OriginClass::Genesis, OriginClass::Admin).unwrap();
    let result = prover.add_transition_with_policy(
        vec![11u8; 32],
        OriginClass::Admin,
        2000,
        proof.path_elements,
        proof.path_indices,
    );
    assert!(result.is_err(), "11th admin transition should fail rate limit");
}