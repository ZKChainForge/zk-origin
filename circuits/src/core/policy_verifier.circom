
pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../lib/poseidon.circom";
include "../lib/constants.circom";

/*
 * Policy Verifier: Merkle Tree-based Policy Enforcement
 */

template PolicyVerifier(MERKLE_DEPTH) {
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input policyRoot;
    
    signal input policyProof[MERKLE_DEPTH];
    signal input policyIndices[MERKLE_DEPTH];
    
    signal output isAllowed;
    
    // ============ VALIDATE ORIGIN CLASSES ============
    component prevCheck = ZKLessThan(8);
    prevCheck.in[0] <== prevOriginClass;
    prevCheck.in[1] <== NUM_ORIGIN_CLASSES();
    prevCheck.out === 1;
    
    component newCheck = ZKLessThan(8);
    newCheck.in[0] <== newOriginClass;
    newCheck.in[1] <== NUM_ORIGIN_CLASSES();
    newCheck.out === 1;
    
    // ============ FOR NOW: ACCEPT ALL TRANSITIONS ============
    // TODO: Implement proper policy merkle verification
    // This is for testing only
    
    isAllowed <== 1;
}
