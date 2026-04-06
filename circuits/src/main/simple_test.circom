pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

template SimpleTest() {
    signal input a;
    signal input b;
    signal output out;
    
    component hasher = Poseidon(2);
    hasher.inputs[0] <== a;
    hasher.inputs[1] <== b;
    
    out <== hasher.out;
}

component main = SimpleTest();
