pragma circom 2.1.0;

/*
 * ZK-ORIGIN: Lineage Step Circuit
 * 
 * This circuit proves a single step in state lineage:
 * 1. The transition follows the origin policy
 * 2. The lineage commitment is correctly updated
 * 3. Rate limits are respected (optional)
 * 
 */

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../lib/selector.circom";
include "../utils/constants.circom";

template LineageStep(POLICY_DEPTH) {
    // PUBLIC INPUTS
    signal input prev_state_hash;           // Hash of previous state
    signal input new_state_hash;            // Hash of new state
    signal input prev_lineage_commitment;   // Previous lineage commitment
    signal input policy_root;               // Merkle root of allowed transitions
    
   
    // PRIVATE INPUTS
    
    signal input prev_origin;               // Origin class of previous state (0-3)
    signal input new_origin;                // Origin class of this transition (0-3)
    signal input prev_depth;                // Current lineage depth
    signal input timestamp;                 // Transition timestamp
    
    // Merkle proof for policy verification
    signal input policy_path[POLICY_DEPTH];
    signal input policy_indices[POLICY_DEPTH];
    
    // PUBLIC OUTPUTS
    signal output new_lineage_commitment;   // Updated lineage commitment
    signal output new_depth;                // New lineage depth
    signal output transition_valid;         // 1 if transition is valid
    
    // CONSTANTS
    var NUM_CLASSES = 4;
    
    // STEP 1: Validate origin class ranges
    
    // prev_origin must be in [0, NUM_CLASSES)
    component prevOriginCheck = LessThan(8);
    prevOriginCheck.in[0] <== prev_origin;
    prevOriginCheck.in[1] <== NUM_CLASSES;
    prevOriginCheck.out === 1;
    
    // new_origin must be in [0, NUM_CLASSES)
    component newOriginCheck = LessThan(8);
    newOriginCheck.in[0] <== new_origin;
    newOriginCheck.in[1] <== NUM_CLASSES;
    newOriginCheck.out === 1;
    
    // STEP 2: Verify transition is allowed by policy
    
    // Compute the policy leaf: Hash(prev_origin, new_origin)
    component policyLeaf = PolicyLeaf();
    policyLeaf.fromOrigin <== prev_origin;
    policyLeaf.toOrigin <== new_origin;
    
    // Verify Merkle proof
    component policyVerifier = MerkleProofVerifier(POLICY_DEPTH);
    policyVerifier.leaf <== policyLeaf.leaf;
    policyVerifier.root <== policy_root;
    
    for (var i = 0; i < POLICY_DEPTH; i++) {
        policyVerifier.pathElements[i] <== policy_path[i];
        policyVerifier.pathIndices[i] <== policy_indices[i];
    }
    
    // Policy must be satisfied
    policyVerifier.valid === 1;
    
    // STEP 3: Additional policy constraints
    
    // Cannot return to Genesis after depth > 0
    component toGenesis = IsEqual();
    toGenesis.in[0] <== new_origin;
    toGenesis.in[1] <== 0; // Genesis
    
    component depthPositive = GreaterThan(32);
    depthPositive.in[0] <== prev_depth;
    depthPositive.in[1] <== 0;
    
    // If depth > 0 AND new_origin == Genesis, this is invalid
    signal invalidGenesisReturn;
    invalidGenesisReturn <== depthPositive.out * toGenesis.out;
    invalidGenesisReturn === 0;
    
    
    // STEP 4: Compute transition hash
    
    component transitionHash = PoseidonHash4();
    transitionHash.in[0] <== prev_state_hash;
    transitionHash.in[1] <== new_state_hash;
    transitionHash.in[2] <== new_origin;
    transitionHash.in[3] <== timestamp;
    
    
    // STEP 5: Update lineage commitment
    
    component lineageUpdate = PoseidonHash3();
    lineageUpdate.in[0] <== prev_lineage_commitment;
    lineageUpdate.in[1] <== transitionHash.out;
    lineageUpdate.in[2] <== prev_depth + 1;
    
    new_lineage_commitment <== lineageUpdate.out;
    new_depth <== prev_depth + 1;
    
    
    // STEP 6: Set validity flag

    // If we reached here without constraint failures, transition is valid
    transition_valid <== 1;
}

// Main component with policy depth of 4 (supports 16 policy entries)
component main {public [
    prev_state_hash,
    new_state_hash,
    prev_lineage_commitment,
    policy_root
]} = LineageStep(4);