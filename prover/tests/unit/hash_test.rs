//! Unit tests for hash functions

use zk_origin::hash::*;

#[test]
fn test_poseidon_hasher_consistency() {
    let hasher = PoseidonHasher::new();
    
    let input = [[1u8; 32], [2u8; 32]];
    
    let hash1 = hasher.hash(&input);
    let hash2 = hasher.hash(&input);
    
    assert_eq!(hash1, hash2);
}

#[test]
fn test_poseidon_different_inputs() {
    let hasher = PoseidonHasher::new();
    
    let hash1 = hasher.hash(&[[1u8; 32]]);
    let hash2 = hasher.hash(&[[2u8; 32]]);
    
    assert_ne!(hash1, hash2);
}

#[test]
fn test_merkle_tree_power_of_two() {
    // 8 leaves (power of 2)
    let leaves: Vec<[u8; 32]> = (0..8)
        .map(|i| {
            let mut arr = [0u8; 32];
            arr[0] = i;
            arr
        })
        .collect();
    
    let tree = MerkleTree::new(leaves);
    
    assert_eq!(tree.depth(), 3); // log2(8) = 3
    assert_eq!(tree.num_leaves(), 8);
}

#[test]
fn test_merkle_tree_non_power_of_two() {
    // 5 leaves (not power of 2, should pad to 8)
    let leaves: Vec<[u8; 32]> = (0..5)
        .map(|i| {
            let mut arr = [0u8; 32];
            arr[0] = i;
            arr
        })
        .collect();
    
    let tree = MerkleTree::new(leaves);
    
    assert_eq!(tree.depth(), 3); // log2(8) = 3
    assert_eq!(tree.num_leaves(), 5);
}

#[test]
fn test_merkle_proof_all_leaves() {
    let leaves: Vec<[u8; 32]> = (0..8)
        .map(|i| {
            let mut arr = [0u8; 32];
            arr[0] = i;
            arr
        })
        .collect();
    
    let tree = MerkleTree::new(leaves);
    
    for i in 0..8 {
        let proof = tree.prove(i).expect("Should generate proof");
        assert!(proof.verify(), "Proof {} should verify", i);
    }
}

#[test]
fn test_merkle_proof_tamper_detection() {
    let leaves: Vec<[u8; 32]> = (0..4)
        .map(|i| {
            let mut arr = [0u8; 32];
            arr[0] = i;
            arr
        })
        .collect();
    
    let tree = MerkleTree::new(leaves);
    let mut proof = tree.prove(0).unwrap();
    
    // Tamper with the leaf
    proof.leaf[0] = 255;
    
    assert!(!proof.verify());
}

#[test]
fn test_policy_tree_construction() {
    use zk_origin::types::OriginClass;
    
    let allowed = vec![
        (OriginClass::Genesis as u8, OriginClass::User as u8),
        (OriginClass::User as u8, OriginClass::User as u8),
        (OriginClass::Admin as u8, OriginClass::User as u8),
        (OriginClass::Admin as u8, OriginClass::Admin as u8),
    ];
    
    let (tree, mapping) = merkle::build_policy_tree(&allowed);
    
    assert_eq!(mapping.len(), 4);
    assert!(tree.num_leaves() >= 4);
    
    // Verify each allowed transition can generate a proof
    for &(from, to) in &allowed {
        let proof = merkle::generate_policy_proof(&tree, &mapping, from, to);
        assert!(proof.is_some(), "Should generate proof for ({}, {})", from, to);
        assert!(proof.unwrap().verify());
    }
}

#[test]
fn test_lineage_commitment_computation() {
    let prev = [1u8; 32];
    let transition = [2u8; 32];
    
    let commit1 = poseidon::compute_lineage_commitment(&prev, &transition, 5);
    let commit2 = poseidon::compute_lineage_commitment(&prev, &transition, 5);
    let commit3 = poseidon::compute_lineage_commitment(&prev, &transition, 6);
    
    assert_eq!(commit1, commit2);
    assert_ne!(commit1, commit3);
}

#[test]
fn test_transition_hash_computation() {
    let prev = [1u8; 32];
    let new = [2u8; 32];
    
    let hash1 = poseidon::compute_transition_hash(&prev, &new, 1, 1000, 0);
    let hash2 = poseidon::compute_transition_hash(&prev, &new, 1, 1000, 0);
    let hash3 = poseidon::compute_transition_hash(&prev, &new, 2, 1000, 0);
    
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
}