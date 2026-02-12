pragma circom 2.1.0;


 //Merkle Tree Verification for ZK-ORIGIN Policy
 // Proves that a (from_origin, to_origin) pair is in the allowed set
 

include "./poseidon.circom";
include "./comparators.circom";

// Verify a Merkle proof
template MerkleProofVerifier(DEPTH) {
    signal input leaf;
    signal input root;
    signal input pathElements[DEPTH];
    signal input pathIndices[DEPTH];
    
    signal output valid;
    
    // Compute root from leaf and path
    component hashers[DEPTH];
    component mux[DEPTH][2];
    
    signal levelHashes[DEPTH + 1];
    levelHashes[0] <== leaf;
    
    for (var i = 0; i < DEPTH; i++) {
        // pathIndices[i] == 0 means leaf is on the left
        // pathIndices[i] == 1 means leaf is on the right
        
        // Ensure pathIndices is binary
        pathIndices[i] * (pathIndices[i] - 1) === 0;
        
        // Select left and right based on index
        mux[i][0] = Mux1();
        mux[i][0].c[0] <== levelHashes[i];
        mux[i][0].c[1] <== pathElements[i];
        mux[i][0].s <== pathIndices[i];
        
        mux[i][1] = Mux1();
        mux[i][1].c[0] <== pathElements[i];
        mux[i][1].c[1] <== levelHashes[i];
        mux[i][1].s <== pathIndices[i];
        
        // Hash left || right
        hashers[i] = PoseidonHash2();
        hashers[i].in[0] <== mux[i][0].out; // left
        hashers[i].in[1] <== mux[i][1].out; // right
        
        levelHashes[i + 1] <== hashers[i].out;
    }
    
    // Check computed root matches expected root
    component eq = IsEqual();
    eq.in[0] <== levelHashes[DEPTH];
    eq.in[1] <== root;
    
    valid <== eq.out;
}

// Compute a leaf for the policy tree
template PolicyLeaf() {
    signal input fromOrigin;
    signal input toOrigin;
    signal output leaf;
    
    component hasher = PoseidonHash2();
    hasher.in[0] <== fromOrigin;
    hasher.in[1] <== toOrigin;
    
    leaf <== hasher.out;
}