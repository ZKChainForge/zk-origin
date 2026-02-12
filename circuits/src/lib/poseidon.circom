pragma circom 2.1.0;


 // This wraps circomlib's Poseidon for our specific use cases
 

include "../../node_modules/circomlib/circuits/poseidon.circom";

// Hash 2 elements (for policy leaf computation)
template PoseidonHash2() {
    signal input in[2];
    signal output out;
    
    component hasher = Poseidon(2);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    
    out <== hasher.out;
}

// Hash 3 elements (for lineage commitment)
template PoseidonHash3() {
    signal input in[3];
    signal output out;
    
    component hasher = Poseidon(3);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    hasher.inputs[2] <== in[2];
    
    out <== hasher.out;
}

// Hash 4 elements (for transition hash)
template PoseidonHash4() {
    signal input in[4];
    signal output out;
    
    component hasher = Poseidon(4);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    hasher.inputs[2] <== in[2];
    hasher.inputs[3] <== in[3];
    
    out <== hasher.out;
}

// Hash 5 elements (for extended transition hash with epoch)
template PoseidonHash5() {
    signal input in[5];
    signal output out;
    
    component hasher = Poseidon(5);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    hasher.inputs[2] <== in[2];
    hasher.inputs[3] <== in[3];
    hasher.inputs[4] <== in[4];
    
    out <== hasher.out;
}