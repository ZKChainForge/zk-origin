pragma circom 2.1.0;

include "./comparators.circom";
include "./poseidon.circom";

// Merkle Proof Verifier
template MerkleProofVerifier(DEPTH) {
    signal input leaf;
    signal input root;
    signal input pathElements[DEPTH];
    signal input pathIndices[DEPTH];
    signal output valid;
    
    signal levelHashes[DEPTH + 1];
    levelHashes[0] <== leaf;
    
    for (var i = 0; i < DEPTH; i++) {
        // Ensure path index is 0 or 1
        pathIndices[i] * (pathIndices[i] - 1) === 0;
        
        signal left;
        signal right;
        
        // Select left and right based on index
        left <== (1 - pathIndices[i]) * levelHashes[i] + pathIndices[i] * pathElements[i];
        right <== pathIndices[i] * levelHashes[i] + (1 - pathIndices[i]) * pathElements[i];
        
        // Hash left and right
        component hasher = PoseidonHash2();
        hasher.in[0] <== left;
        hasher.in[1] <== right;
        levelHashes[i + 1] <== hasher.out;
    }
    
    // Check final hash matches root
    component eq = IsEqual();
    eq.in[0] <== levelHashes[DEPTH];
    eq.in[1] <== root;
    valid <== eq.out;
}

// Compute Merkle leaf for policy transition
template PolicyLeaf() {
    signal input fromOrigin;
    signal input toOrigin;
    signal output leaf;
    
    component hasher = PoseidonHash2();
    hasher.in[0] <== fromOrigin;
    hasher.in[1] <== toOrigin;
    leaf <== hasher.out;
}

// Verify policy membership (transition allowed by policy)
template VerifyPolicyMembership(DEPTH) {
    signal input fromOrigin;
    signal input toOrigin;
    signal input policyRoot;
    signal input pathElements[DEPTH];
    signal input pathIndices[DEPTH];
    signal output isAllowed;
    
    component leaf = PolicyLeaf();
    leaf.fromOrigin <== fromOrigin;
    leaf.toOrigin <== toOrigin;
    
    component merkleVerifier = MerkleProofVerifier(DEPTH);
    merkleVerifier.leaf <== leaf.leaf;
    merkleVerifier.root <== policyRoot;
    for (var i = 0; i < DEPTH; i++) {
        merkleVerifier.pathElements[i] <== pathElements[i];
        merkleVerifier.pathIndices[i] <== pathIndices[i];
    }
    
    isAllowed <== merkleVerifier.valid;
}

// Build Merkle root from leaves
template MerkleRoot(DEPTH) {
    signal input leaves[2**DEPTH];
    signal output root;
    
    signal levelValues[DEPTH + 1][2**(DEPTH + 1)];
    
    // Initialize with leaves
    for (var j = 0; j < 2**DEPTH; j++) {
        levelValues[0][j] <== leaves[j];
    }
    
    // Build tree bottom-up
    for (var level = 0; level < DEPTH; level++) {
        for (var i = 0; i < 2**(DEPTH - level - 1); i++) {
            component hasher = PoseidonHash2();
            hasher.in[0] <== levelValues[level][2*i];
            hasher.in[1] <== levelValues[level][2*i + 1];
            levelValues[level + 1][i] <== hasher.out;
        }
    }
    
    root <== levelValues[DEPTH][0];
}