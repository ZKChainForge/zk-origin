pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";

// Simplified recursive lineage (no actual recursion yet)
template RecursiveLineageMain() {
    signal input prevStateHash;
    signal input newStateHash;
    signal input prevLineageCommitment;
    signal input depth;
    signal input originClass;
    
    signal output newLineageCommitment;
    signal output newDepth;
    
    // Verify depth is reasonable
    component depthCheck = LessThan(32);
    depthCheck.in[0] <== depth;
    depthCheck.in[1] <== 1000000;
    depthCheck.out === 1;
    
    // Compute transition hash
    component transitionHasher = PoseidonHash3();
    transitionHasher.in[0] <== prevStateHash;
    transitionHasher.in[1] <== newStateHash;
    transitionHasher.in[2] <== originClass;
    
    // Update lineage commitment
    component lineageHasher = PoseidonHash3();
    lineageHasher.in[0] <== prevLineageCommitment;
    lineageHasher.in[1] <== transitionHasher.out;
    lineageHasher.in[2] <== depth + 1;
    
    newLineageCommitment <== lineageHasher.out;
    newDepth <== depth + 1;
}

component main {public [prevStateHash, newStateHash, prevLineageCommitment]} = RecursiveLineageMain();
