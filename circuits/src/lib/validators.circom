pragma circom 2.1.0;

include "./comparators.circom";
include "./constants.circom";

/*
 * Input Validators
 * 
 * Ensures all inputs are within valid ranges.
 */

// ============================================
// VALID ORIGIN CLASS (0-6)
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
// VALID DEPTH (< MAX_LINEAGE_DEPTH)
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
// VALID TIMESTAMP (< u32::MAX)
// ============================================
template ValidTimestamp() {
    signal input timestamp;
    signal output valid;
    
    component lt = ZKLessThan(32);
    lt.in[0] <== timestamp;
    lt.in[1] <== COUNTER_MAX();
    valid <== lt.out;
}

// ============================================
// VALID EPOCH (< MAX_EPOCH_NUMBER)
// ============================================
template ValidEpoch() {
    signal input epoch;
    signal output valid;
    
    component lt = ZKLessThan(32);
    lt.in[0] <== epoch;
    lt.in[1] <== MAX_EPOCH_NUMBER();
    valid <== lt.out;
}

// ============================================
// VALID NONCE (< 2^64)
// ============================================
template ValidNonce() {
    signal input nonce;
    signal output valid;
    
    component lt = ZKLessThan(64);
    lt.in[0] <== nonce;
    lt.in[1] <== 18446744073709551615; // 2^64 - 1
    valid <== lt.out;
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
// BINARY CONSTRAINT (0 or 1)
// ============================================
template IsBinary() {
    signal input value;
    signal output valid;
    
    value * (value - 1) === 0;
    valid <== 1;
}