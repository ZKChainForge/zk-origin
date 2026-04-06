pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

// 2-input Poseidon hash
template PoseidonHash2() {
    signal input in[2];
    signal output out;
    component hasher = Poseidon(2);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    out <== hasher.out;
}

// 3-input Poseidon hash
template PoseidonHash3() {
    signal input in[3];
    signal output out;
    component hasher = Poseidon(3);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    hasher.inputs[2] <== in[2];
    out <== hasher.out;
}

// 4-input Poseidon hash
template PoseidonHash4() {
    signal input in[4];
    signal output out;
    component hasher = Poseidon(4);
    for (var i = 0; i < 4; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// 5-input Poseidon hash
template PoseidonHash5() {
    signal input in[5];
    signal output out;
    component hasher = Poseidon(5);
    for (var i = 0; i < 5; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// 6-input Poseidon hash
template PoseidonHash6() {
    signal input in[6];
    signal output out;
    component hasher = Poseidon(6);
    for (var i = 0; i < 6; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// 7-input Poseidon hash
template PoseidonHash7() {
    signal input in[7];
    signal output out;
    component hasher = Poseidon(7);
    for (var i = 0; i < 7; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// 8-input Poseidon hash
template PoseidonHash8() {
    signal input in[8];
    signal output out;
    component hasher = Poseidon(8);
    for (var i = 0; i < 8; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// N-input Poseidon hash (generic)
template PoseidonHashN(N) {
    signal input in[N];
    signal output out;
    component hasher = Poseidon(N);
    for (var i = 0; i < N; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}