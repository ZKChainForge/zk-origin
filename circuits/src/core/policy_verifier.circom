pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../lib/poseidon.circom";
include "../lib/constants.circom";

/*
 * Policy Verifier: Merkle Tree-based Policy Enforcement
 * 
 * PRODUCTION VERSION - Actually verifies transitions via Merkle proof
 */

template PolicyVerifier(MERKLE_DEPTH) {
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input policyRoot;
    
    signal input policyProof[MERKLE_DEPTH];
    signal input policyIndices[MERKLE_DEPTH];
    
    signal output isAllowed;
    
    // ============ STEP 1: VALIDATE ORIGIN CLASSES ============
    component prevCheck = ZKLessThan(8);
    prevCheck.in[0] <== prevOriginClass;
    prevCheck.in[1] <== NUM_ORIGIN_CLASSES();
    prevCheck.out === 1;
    
    component newCheck = ZKLessThan(8);
    newCheck.in[0] <== newOriginClass;
    newCheck.in[1] <== NUM_ORIGIN_CLASSES();
    newCheck.out === 1;
    
    // ============ STEP 2: COMPUTE TRANSITION LEAF ============
    component leafHasher = PoseidonHash2();
    leafHasher.in[0] <== prevOriginClass;
    leafHasher.in[1] <== newOriginClass;
    
    // ============ STEP 3: VERIFY MERKLE PROOF ============
    component merkleVerifier = MerkleProofVerifier(MERKLE_DEPTH);
    merkleVerifier.leaf <== leafHasher.out;
    merkleVerifier.root <== policyRoot;
    
    for (var i = 0; i < MERKLE_DEPTH; i++) {
        merkleVerifier.pathElements[i] <== policyProof[i];
        merkleVerifier.pathIndices[i] <== policyIndices[i];
    }
    
    // ============ ENFORCE MERKLE VERIFICATION ============
    merkleVerifier.valid === 1;
    isAllowed <== merkleVerifier.valid;
}