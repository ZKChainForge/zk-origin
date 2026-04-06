pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

template PoseidonHash2() {
    signal input in[2];
    signal output out;
    component hasher = Poseidon(2);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    out <== hasher.out;
}

template PoseidonHash3() {
    signal input in[3];
    signal output out;
    component hasher = Poseidon(3);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    hasher.inputs[2] <== in[2];
    out <== hasher.out;
}

template PoseidonHash4() {
    signal input in[4];
    signal output out;
    component hasher = Poseidon(4);
    for (var i = 0; i < 4; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

template PoseidonHash5() {
    signal input in[5];
    signal output out;
    component hasher = Poseidon(5);
    for (var i = 0; i < 5; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

template PoseidonHash7() {
    signal input in[7];
    signal output out;
    component hasher = Poseidon(7);
    for (var i = 0; i < 7; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}
