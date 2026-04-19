pragma circom 2.1.0;

include "./poseidon.circom";
include "./comparators.circom";

/**
 * @title Merkle Tree Operations (PRODUCTION)
 * @notice Standard Merkle proof verification and updates
 * 
 * SECURITY:
 *  Proper leaf ordering with path indices
 *  Prevents ordering attacks
 *  Constraints all operations
 * 
 * PRODUCTION NOTES:
 * - All tree operations use Poseidon hashing
 * - Path indices must be binary (0 or 1)
 * - Merkle proofs are non-interactive
 * 
 * CONSTRAINTS (for depth D):
 * - MerkleProofVerifier: ~300*D constraints
 * - MerkleRoot: ~300*N constraints (N = num leaves)
 * - SparseMerkleUpdate: ~600*D constraints
 */

// ============================================
// MERKLE PROOF VERIFIER
// ============================================
template MerkleProofVerifier(DEPTH) {
    signal input leaf;
    signal input root;
    signal input pathElements[DEPTH];
    signal input pathIndices[DEPTH];
    signal output valid;
    
    signal levelHashes[DEPTH + 1];
    levelHashes[0] <== leaf;
    
    // Components for each level
    component hashers[DEPTH];
    component muxLefts[DEPTH];
    component muxRights[DEPTH];
    signal lefts[DEPTH];
    signal rights[DEPTH];
    
    // Verify path indices are binary
    component indexValidators[DEPTH];
    for (var i = 0; i < DEPTH; i++) {
        indexValidators[i] = IsBinary();
        indexValidators[i].value <== pathIndices[i];
        indexValidators[i].valid === 1;
    }
    
    // Iterate up the tree
    for (var i = 0; i < DEPTH; i++) {
        // Ensure path index is 0 or 1
        pathIndices[i] * (pathIndices[i] - 1) === 0;
        
        // Use multiplexer to select left and right
        // if pathIndices[i] == 0: left = currentLevel, right = pathElement
        // if pathIndices[i] == 1: left = pathElement, right = currentLevel
        muxLefts[i] = ZKMux1();
        muxLefts[i].c[0] <== levelHashes[i];
        muxLefts[i].c[1] <== pathElements[i];
        muxLefts[i].s <== pathIndices[i];
        lefts[i] <== muxLefts[i].out;
        
        muxRights[i] = ZKMux1();
        muxRights[i].c[0] <== pathElements[i];
        muxRights[i].c[1] <== levelHashes[i];
        muxRights[i].s <== pathIndices[i];
        rights[i] <== muxRights[i].out;
        
        // Hash left and right
        hashers[i] = PoseidonHash2();
        hashers[i].in[0] <== lefts[i];
        hashers[i].in[1] <== rights[i];
        levelHashes[i + 1] <== hashers[i].out;
    }
    
    // Final check: computed root must match expected root
    component rootCheck = ZKIsEqual();
    rootCheck.in[0] <== levelHashes[DEPTH];
    rootCheck.in[1] <== root;
    valid <== rootCheck.out;
}

// ============================================
// MERKLE ROOT COMPUTATION
// ============================================
template MerkleRoot(DEPTH) {
    signal input leaves[2**DEPTH];
    signal output root;
    
    // Declare all hashes upfront (circom requirement)
    signal levelValues[DEPTH + 1][2**(DEPTH + 1)];
    
    // Initialize with leaves
    for (var j = 0; j < 2**DEPTH; j++) {
        levelValues[0][j] <== leaves[j];
    }
    
    // Declare hashers array
    component hashersArray[DEPTH][2**(DEPTH)];
    
    // Build tree bottom-up
    for (var level = 0; level < DEPTH; level++) {
        for (var i = 0; i < 2**(DEPTH - level - 1); i++) {
            hashersArray[level][i] = PoseidonHash2();
            hashersArray[level][i].in[0] <== levelValues[level][2*i];
            hashersArray[level][i].in[1] <== levelValues[level][2*i + 1];
            levelValues[level + 1][i] <== hashersArray[level][i].out;
        }
    }
    
    root <== levelValues[DEPTH][0];
}

// ============================================
// SPARSE MERKLE TREE UPDATE
// ============================================
template SparseMerkleUpdate(DEPTH) {
    signal input oldLeaf;
    signal input newLeaf;
    signal input oldRoot;
    signal input pathElements[DEPTH];
    signal input pathIndices[DEPTH];
    signal output newRoot;
    
    // Verify old leaf with old root
    component oldVerifier = MerkleProofVerifier(DEPTH);
    oldVerifier.leaf <== oldLeaf;
    oldVerifier.root <== oldRoot;
    for (var i = 0; i < DEPTH; i++) {
        oldVerifier.pathElements[i] <== pathElements[i];
        oldVerifier.pathIndices[i] <== pathIndices[i];
    }
    oldVerifier.valid === 1;
    
    // Compute new root with new leaf
    signal levelHashes[DEPTH + 1];
    levelHashes[0] <== newLeaf;
    
    // Components
    component hashers[DEPTH];
    component muxLefts[DEPTH];
    component muxRights[DEPTH];
    signal lefts[DEPTH];
    signal rights[DEPTH];
    
    // Reconstruct tree with new leaf
    for (var i = 0; i < DEPTH; i++) {
        // Use multiplexer
        muxLefts[i] = ZKMux1();
        muxLefts[i].c[0] <== levelHashes[i];
        muxLefts[i].c[1] <== pathElements[i];
        muxLefts[i].s <== pathIndices[i];
        lefts[i] <== muxLefts[i].out;
        
        muxRights[i] = ZKMux1();
        muxRights[i].c[0] <== pathElements[i];
        muxRights[i].c[1] <== levelHashes[i];
        muxRights[i].s <== pathIndices[i];
        rights[i] <== muxRights[i].out;
        
        // Hash
        hashers[i] = PoseidonHash2();
        hashers[i].in[0] <== lefts[i];
        hashers[i].in[1] <== rights[i];
        levelHashes[i + 1] <== hashers[i].out;
    }
    
    newRoot <== levelHashes[DEPTH];
}

// ============================================
// MERKLE INCLUSION PROOF (alternative name)
// ============================================
template MerkleInclusionProof(DEPTH) {
    signal input leaf;
    signal input root;
    signal input proof[DEPTH];
    signal input indices[DEPTH];
    signal output included;
    
    component verifier = MerkleProofVerifier(DEPTH);
    verifier.leaf <== leaf;
    verifier.root <== root;
    for (var i = 0; i < DEPTH; i++) {
        verifier.pathElements[i] <== proof[i];
        verifier.pathIndices[i] <== indices[i];
    }
    
    included <== verifier.valid;
}