pragma circom 2.1.0;

//These are helper templates for type checking in circuits
 

include "../lib/comparators.circom";

// Validate that a value is a valid origin class (0-3)
template ValidOriginClass() {
    signal input origin;
    signal output valid;
    
    component lt = LessThan(8);
    lt.in[0] <== origin;
    lt.in[1] <== 4; // NUM_ORIGIN_CLASSES
    
    valid <== lt.out;
}

// Validate that a depth value is reasonable (not overflowed)
template ValidDepth() {
    signal input depth;
    signal output valid;
    
    component lt = LessThan(32);
    lt.in[0] <== depth;
    lt.in[1] <== 4294967295; // MAX_LINEAGE_DEPTH
    
    valid <== lt.out;
}