pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

template PoseidonHash2() {
    signal input in[2];
    signal output out;
    component h = Poseidon(2);
    h.inputs[0] <== in[0];
    h.inputs[1] <== in[1];
    out <== h.out;
}

template PoseidonHash3() {
    signal input in[3];
    signal output out;
    component h = Poseidon(3);
    for (var i = 0; i < 3; i++) h.inputs[i] <== in[i];
    out <== h.out;
}

template PoseidonHash4() {
    signal input in[4];
    signal output out;
    component h = Poseidon(4);
    for (var i = 0; i < 4; i++) h.inputs[i] <== in[i];
    out <== h.out;
}

template PoseidonHash5() {
    signal input in[5];
    signal output out;
    component h = Poseidon(5);
    for (var i = 0; i < 5; i++) h.inputs[i] <== in[i];
    out <== h.out;
}

template PoseidonHash6() {
    signal input in[6];
    signal output out;
    component h = Poseidon(6);
    for (var i = 0; i < 6; i++) h.inputs[i] <== in[i];
    out <== h.out;
}

template PoseidonHash8() {
    signal input in[8];
    signal output out;
    component h = Poseidon(8);
    for (var i = 0; i < 8; i++) h.inputs[i] <== in[i];
    out <== h.out;
}