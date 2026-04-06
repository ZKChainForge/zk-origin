pragma circom 2.1.0;

include "./constants.circom";
include "../lib/comparators.circom";

template ValidOriginClass() {
    signal input origin;
    signal output valid;
    component lt = LessThan(8);
    lt.in[0] <== origin;
    lt.in[1] <== NUM_ORIGIN_CLASSES();
    valid <== lt.out;
}

template ValidDepth() {
    signal input depth;
    signal output valid;
    component lt = LessThan(32);
    lt.in[0] <== depth;
    lt.in[1] <== MAX_LINEAGE_DEPTH();
    valid <== lt.out;
}

template ValidTimestamp() {
    signal input timestamp;
    signal output valid;
    component lt = LessThan(32);
    lt.in[0] <== timestamp;
    lt.in[1] <== 4294967295;
    valid <== lt.out;
}

template ValidEpoch() {
    signal input epoch;
    signal output valid;
    component lt = LessThan(32);
    lt.in[0] <== epoch;
    lt.in[1] <== MAX_EPOCH_NUMBER();
    valid <== lt.out;
}

template ValidHash() {
    signal input hash;
    signal output valid;
    component isNonZero = IsZero();
    isNonZero.in <== hash;
    valid <== 1 - isNonZero.out;
}

template DifferentHashes() {
    signal input hash1;
    signal input hash2;
    signal output valid;
    component eq = IsEqual();
    eq.in[0] <== hash1;
    eq.in[1] <== hash2;
    valid <== 1 - eq.out;
}
