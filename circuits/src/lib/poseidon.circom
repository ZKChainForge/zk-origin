pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

/*
 * Poseidon Hash Wrappers
 * 
 * ZK-friendly hashing (~300 constraints per hash).
 * Requires circomlib dependency.
 */

// ============================================
// 2-INPUT POSEIDON HASH
// ============================================
template PoseidonHash2() {
    signal input in[2];
    signal output out;
    
    component hasher = Poseidon(2);
    hasher.inputs[0] <== in[0];
    hasher.inputs[1] <== in[1];
    out <== hasher.out;
}

// ============================================
// 3-INPUT POSEIDON HASH
// ============================================
template PoseidonHash3() {
    signal input in[3];
    signal output out;
    
    component hasher = Poseidon(3);
    for (var i = 0; i < 3; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// ============================================
// 4-INPUT POSEIDON HASH
// ============================================
template PoseidonHash4() {
    signal input in[4];
    signal output out;
    
    component hasher = Poseidon(4);
    for (var i = 0; i < 4; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// ============================================
// 5-INPUT POSEIDON HASH
// ============================================
template PoseidonHash5() {
    signal input in[5];
    signal output out;
    
    component hasher = Poseidon(5);
    for (var i = 0; i < 5; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// ============================================
// 6-INPUT POSEIDON HASH
// ============================================
template PoseidonHash6() {
    signal input in[6];
    signal output out;
    
    component hasher = Poseidon(6);
    for (var i = 0; i < 6; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// ============================================
// 7-INPUT POSEIDON HASH (for counters)
// ============================================
template PoseidonHash7() {
    signal input in[7];
    signal output out;
    
    component hasher = Poseidon(7);
    for (var i = 0; i < 7; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// ============================================
// 8-INPUT POSEIDON HASH
// ============================================
template PoseidonHash8() {
    signal input in[8];
    signal output out;
    
    component hasher = Poseidon(8);
    for (var i = 0; i < 8; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}

// ============================================
// N-INPUT POSEIDON HASH (generic)
// ============================================
template PoseidonHashN(N) {
    signal input in[N];
    signal output out;
    
    component hasher = Poseidon(N);
    for (var i = 0; i < N; i++) {
        hasher.inputs[i] <== in[i];
    }
    out <== hasher.out;
}