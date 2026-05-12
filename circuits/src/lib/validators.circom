pragma circom 2.1.0;

include "./comparators.circom";
include "./constants.circom";
include "../../node_modules/circomlib/circuits/bitify.circom";

// ============================================
// VALID ORIGIN CLASS
// ============================================
template ValidOriginClass() {
    signal input origin;
    signal output valid;
    
    component lt = ZKLessThan(8);
    lt.in[0] <== origin;
    lt.in[1] <== NUM_ORIGIN_CLASSES();
    valid <== lt.out;
}

// ============================================
// VALID DEPTH
// ============================================
template ValidDepth() {
    signal input depth;
    signal output valid;
    
    component lt = ZKLessThan(32);
    lt.in[0] <== depth;
    lt.in[1] <== MAX_LINEAGE_DEPTH();
    valid <== lt.out;
}

// ============================================
// VALID NONCE
// ============================================
template ValidNonce() {
    signal input nonce;
    signal output valid;
    
    component lt = ZKLessThan(64);
    lt.in[0] <== nonce;
    lt.in[1] <== MAX_NONCE();
    valid <== lt.out;
}

// ============================================
// BINARY CONSTRAINT (ONLY DEFINITION HERE)
// ============================================
template IsBinary() {
    signal input value;
    signal output valid;
    
    value * (value - 1) === 0;
    valid <== 1;
}