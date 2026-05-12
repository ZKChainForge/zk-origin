pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/comparators.circom";
include "../../node_modules/circomlib/circuits/bitify.circom";

// ============================================
// IS ZERO
// ============================================
template ZKIsZero() {
    signal input in;
    signal output out;
    signal inv;
    
    inv <-- in != 0 ? 1 / in : 0;
    out <== 1 - in * inv;
    
    in * out === 0;
}

// ============================================
// IS EQUAL
// ============================================
template ZKIsEqual() {
    signal input in[2];
    signal output out;
    
    component isz = ZKIsZero();
    isz.in <== in[1] - in[0];
    out <== isz.out;
}

// ============================================
// LESS THAN (uses n bits)
// ============================================
template ZKLessThan(n) {
    assert(n <= 252);
    signal input in[2];
    signal output out;
    
    component lt = LessThan(n);
    lt.in[0] <== in[0];
    lt.in[1] <== in[1];
    out <== lt.out;
}

// ============================================
// GREATER THAN
// ============================================
template ZKGreaterThan(n) {
    assert(n <= 252);
    signal input in[2];
    signal output out;
    
    component lt = LessThan(n);
    lt.in[0] <== in[1];
    lt.in[1] <== in[0];
    out <== lt.out;
}

// ============================================
// LESS THAN OR EQUAL (overflow safe)
// ============================================
template ZKLessEqThan(n) {
    assert(n <= 251);
    signal input in[2];
    signal output out;
    
    component lt = LessThan(n+1);
    lt.in[0] <== in[0];
    lt.in[1] <== in[1] + 1;
    out <== lt.out;
}

// ============================================
// GREATER THAN OR EQUAL (overflow safe)
// ============================================
template ZKGreaterEqThan(n) {
    assert(n <= 251);
    signal input in[2];
    signal output out;
    
    component lt = LessThan(n+1);
    lt.in[0] <== in[1];
    lt.in[1] <== in[0] + 1;
    out <== lt.out;
}

// ============================================
// TERNARY MUX
// ============================================
template ZKMux1() {
    signal input c[2];
    signal input s;
    signal output out;
    
    s * (s - 1) === 0;
    out <== c[0] + s * (c[1] - c[0]);
}

