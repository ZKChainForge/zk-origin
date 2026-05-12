pragma circom 2.1.0;

include "./poseidon.circom";
include "./comparators.circom";
include "./validators.circom";  // Import IsBinary from here

template MerkleProofVerifier(DEPTH) {
    signal input leaf;
    signal input root;
    signal input pathElements[DEPTH];
    signal input pathIndices[DEPTH];
    signal output valid;
    
    signal levelHashes[DEPTH + 1];
    levelHashes[0] <== leaf;
    
    component hashers[DEPTH];
    component muxLefts[DEPTH];
    component muxRights[DEPTH];
    
    component indexValidators[DEPTH];
    for (var i = 0; i < DEPTH; i++) {
        indexValidators[i] = IsBinary();
        indexValidators[i].value <== pathIndices[i];
        indexValidators[i].valid === 1;
    }
    
    for (var i = 0; i < DEPTH; i++) {
        muxLefts[i] = ZKMux1();
        muxLefts[i].c[0] <== levelHashes[i];
        muxLefts[i].c[1] <== pathElements[i];
        muxLefts[i].s <== pathIndices[i];
        
        muxRights[i] = ZKMux1();
        muxRights[i].c[0] <== pathElements[i];
        muxRights[i].c[1] <== levelHashes[i];
        muxRights[i].s <== pathIndices[i];
        
        hashers[i] = PoseidonHash2();
        hashers[i].in[0] <== muxLefts[i].out;
        hashers[i].in[1] <== muxRights[i].out;
        levelHashes[i + 1] <== hashers[i].out;
    }
    
    component rootCheck = ZKIsEqual();
    rootCheck.in[0] <== levelHashes[DEPTH];
    rootCheck.in[1] <== root;
    valid <== rootCheck.out;
}