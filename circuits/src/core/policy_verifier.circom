pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../lib/poseidon.circom";
include "../lib/constants.circom";

/**
 * @title Policy Verifier (PRODUCTION)
 * @notice Merkle tree-based policy enforcement
 * 
 * SECURITY:
 *  Origin classes validated (0-6)
 *  Transition must be in allowed set (Merkle proof)
 *  Merkle root must match contract policy
 *  Prevents unauthorized transitions
 *  Deterministic verification
 * 
 * PROTECTION: POLICY PROTECTED
 * - Only transitions in policy Merkle tree allowed
 * - Policy is cryptographically enforced
 * - Prevents privilege escalation
 * - Prevents lateral movement
 * 
 * INPUT AUTHORIZATION:
 * - prevOriginClass: Current origin (0-6)
 * - newOriginClass: Attempted next origin (0-6)
 * - policyRoot: Merkle root of allowed transitions
 * - policyProof[DEPTH]: Merkle path
 * - policyIndices[DEPTH]: Path direction bits
 * 
 * OUTPUT GUARANTEE:
 * - isAllowed: 1 if transition in policy, circuit fails if not
 * 
 * CONSTRAINTS: ~300*POLICY_MERKLE_DEPTH
 * Example: 6 levels = ~1800 constraints
 * 
 * PRODUCTION CHECKLIST:
 *  Both origin classes validated (< 7)
 *  Transition leaf computed
 *  Merkle proof verified against root
 *  Proof fails entire circuit if invalid
 *  No unconstrained fallback
 */

template PolicyVerifier(MERKLE_DEPTH) {
    // ============ PUBLIC INPUTS ============
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input policyRoot;
    
    // ============ PRIVATE INPUTS ============
    signal input policyProof[MERKLE_DEPTH];
    signal input policyIndices[MERKLE_DEPTH];
    
    // ============ PUBLIC OUTPUTS ============
    signal output isAllowed;
    
    // ============ STEP 1: VALIDATE ORIGIN CLASSES ============
    component prevCheck = ZKLessThan(8);
    prevCheck.in[0] <== prevOriginClass;
    prevCheck.in[1] <== NUM_ORIGIN_CLASSES();
    prevCheck.out === 1;  // ENFORCE: valid origin
    
    component newCheck = ZKLessThan(8);
    newCheck.in[0] <== newOriginClass;
    newCheck.in[1] <== NUM_ORIGIN_CLASSES();
    newCheck.out === 1;  // ENFORCE: valid origin
    
    // ============ STEP 2: COMPUTE TRANSITION LEAF ============
    // Leaf = Hash(prevOriginClass, newOriginClass)
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
    
    // ENFORCE: Merkle proof MUST be valid
    merkleVerifier.valid === 1;
    isAllowed <== 1;
}