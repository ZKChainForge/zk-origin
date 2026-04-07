pragma circom 2.1.0;

/*
 * Core Comparison Circuits (Custom)
 * 
 * Safe constraint implementations for common comparisons.
 * Renamed to avoid conflicts with circomlib comparators.
 */

include "../../node_modules/circomlib/circuits/comparators.circom";
include "../../node_modules/circomlib/circuits/bitify.circom";

// ============================================
// IS ZERO (Custom Implementation)
// ============================================
template ZKIsZero() {
    signal input in;
    signal output out;
    signal inv;
    
    inv <-- in != 0 ? 1 / in : 0;
    out <-- in == 0 ? 1 : 0;
    
    in * inv + out === 1;
    out * (out - 1) === 0;
}

// ============================================
// IS EQUAL (Custom Implementation)
// ============================================
template ZKIsEqual() {
    signal input in[2];
    signal output out;
    
    component isz = ZKIsZero();
    isz.in <== in[1] - in[0];
    out <== isz.out;
}

// ============================================
// LESS THAN (Wrapper for circomlib)
// ============================================
template ZKLessThan(n) {
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
    signal input in[2];
    signal output out;
    
    component lt = LessThan(n);
    lt.in[0] <== in[1];
    lt.in[1] <== in[0];
    out <== lt.out;
}

// ============================================
// LESS THAN OR EQUAL
// ============================================
template ZKLessEqThan(n) {
    signal input in[2];
    signal output out;
    
    component lt = LessThan(n);
    lt.in[0] <== in[0];
    lt.in[1] <== in[1] + 1;
    out <== lt.out;
}

// ============================================
// GREATER THAN OR EQUAL
// ============================================
template ZKGreaterEqThan(n) {
    signal input in[2];
    signal output out;
    
    component lt = LessThan(n);
    lt.in[0] <== in[1];
    lt.in[1] <== in[0] + 1;
    out <== lt.out;
}

// ============================================
// IN RANGE [min, max]
// ============================================
template ZKInRange(n) {
    signal input value;
    signal input min;
    signal input max;
    signal output out;
    
    component gtEq = ZKGreaterEqThan(n);
    gtEq.in[0] <== value;
    gtEq.in[1] <== min;
    
    component ltEq = ZKLessEqThan(n);
    ltEq.in[0] <== value;
    ltEq.in[1] <== max;
    
    out <== gtEq.out * ltEq.out;
}

// ============================================
// BITWISE AND
// ============================================
template ZKBitwiseAnd() {
    signal input in[2];
    signal output out;
    out <== in[0] * in[1];
}

// ============================================
// BITWISE OR
// ============================================
template ZKBitwiseOr() {
    signal input in[2];
    signal output out;
    out <== in[0] + in[1] - in[0] * in[1];
}

// ============================================
// BITWISE NOT
// ============================================
template ZKBitwiseNot() {
    signal input in;
    signal output out;
    out <== 1 - in;
}

// ============================================
// BITWISE XOR
// ============================================
template ZKBitwiseXor() {
    signal input in[2];
    signal output out;
    out <== in[0] + in[1] - 2 * in[0] * in[1];
}

// ============================================
// MULTIPLEXER (2-to-1 selector)
// ============================================
template ZKMux1() {
    signal input c[2];
    signal input s;
    signal output out;
    out <== c[0] + s * (c[1] - c[0]);
}

// ============================================
// MULTIPLEXER (4-to-1 selector)
// ============================================
template ZKMux4() {
    signal input c[4];
    signal input s[2];
    signal output out;
    
    component mux1 = ZKMux1();
    mux1.c[0] <== c[0];
    mux1.c[1] <== c[1];
    mux1.s <== s[0];
    
    component mux2 = ZKMux1();
    mux2.c[0] <== c[2];
    mux2.c[1] <== c[3];
    mux2.s <== s[0];
    
    component mux3 = ZKMux1();
    mux3.c[0] <== mux1.out;
    mux3.c[1] <== mux2.out;
    mux3.s <== s[1];
    
    out <== mux3.out;
}