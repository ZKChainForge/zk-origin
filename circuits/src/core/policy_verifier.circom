pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../lib/poseidon.circom";

// Verify that a transition is allowed by policy
template PolicyVerifier(MERKLE_DEPTH) {
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input policyRoot;
    signal input policyProof[MERKLE_DEPTH];
    signal input policyIndices[MERKLE_DEPTH];
    signal output isAllowed;
    
    // Verify both origin classes are valid
    component prevCheck = LessThan(8);
    prevCheck.in[0] <== prevOriginClass;
    prevCheck.in[1] <== 6;  // NUM_ORIGIN_CLASSES
    prevCheck.out === 1;
    
    component newCheck = LessThan(8);
    newCheck.in[0] <== newOriginClass;
    newCheck.in[1] <== 6;
    newCheck.out === 1;
    
    // Create leaf for (from, to) pair
    component leaf = PolicyLeaf();
    leaf.fromOrigin <== prevOriginClass;
    leaf.toOrigin <== newOriginClass;
    
    // Verify leaf is in policy tree
    component merkleVerifier = MerkleProofVerifier(MERKLE_DEPTH);
    merkleVerifier.leaf <== leaf.leaf;
    merkleVerifier.root <== policyRoot;
    for (var i = 0; i < MERKLE_DEPTH; i++) {
        merkleVerifier.pathElements[i] <== policyProof[i];
        merkleVerifier.pathIndices[i] <== policyIndices[i];
    }
    
    isAllowed <== merkleVerifier.valid;
    merkleVerifier.valid === 1;  // Transition must be allowed
}

component main {public [
    prevOriginClass,
    newOriginClass,
    policyRoot
]} = PolicyVerifier(6);