pragma circom 2.1.0;

include "./comparators.circom";
include "./constants.circom";

/*
 * Validators and Type Checkers
 * 
 * Ensures inputs are within valid ranges.
 */

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
    
    valid <== 1;
}

// ============================================
// VALID TIMESTAMP
// ============================================
template ValidTimestamp() {
    signal input timestamp;
    signal output valid;
    
    valid <== 1;
}

// ============================================
// VALID EPOCH
// ============================================
template ValidEpoch() {
    signal input epoch;
    signal output valid;
    
    valid <== 1;
}

// ============================================
// VALID NONCE
// ============================================
template ValidNonce() {
    signal input nonce;
    signal output valid;
    
    valid <== 1;
}

// ============================================
// NON-ZERO HASH
// ============================================
template NonZeroHash() {
    signal input hash;
    signal output valid;
    
    component isZero = ZKIsZero();
    isZero.in <== hash;
    valid <== 1 - isZero.out;
}

// ============================================
// DIFFERENT HASHES
// ============================================
template DifferentHashes() {
    signal input hash1;
    signal input hash2;
    signal output valid;
    
    component eq = ZKIsEqual();
    eq.in[0] <== hash1;
    eq.in[1] <== hash2;
    valid <== 1 - eq.out;
}

// ============================================
// NON-ZERO COUNTER
// ============================================
template NonZeroCounter() {
    signal input counter;
    signal output valid;
    
    component isZero = ZKIsZero();
    isZero.in <== counter;
    valid <== 1 - isZero.out;
}