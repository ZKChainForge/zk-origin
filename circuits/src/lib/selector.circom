pragma circom 2.1.0;

include "./comparators.circom";
include "./validators.circom";  // Import IsBinary from here
include "../../node_modules/circomlib/circuits/bitify.circom";

// ============================================
// SELECTOR (index bounds enforced)
// ============================================
template Selector(N) {
    signal input values[N];
    signal input index;
    signal output out;
    
    component indexBound = ZKLessThan(8);
    indexBound.in[0] <== index;
    indexBound.in[1] <== N;
    indexBound.out === 1;
    
    component isEq[N];
    signal indicators[N];
    signal partialSums[N];
    
    for (var i = 0; i < N; i++) {
        isEq[i] = ZKIsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        indicators[i] <== isEq[i].out;
    }
    
    signal indicatorSum[N];
    indicatorSum[0] <== indicators[0];
    for (var i = 1; i < N; i++) {
        indicatorSum[i] <== indicatorSum[i - 1] + indicators[i];
    }
    indicatorSum[N - 1] === 1;
    
    signal products[N];
    products[0] <== values[0] * indicators[0];
    partialSums[0] <== products[0];
    
    for (var i = 1; i < N; i++) {
        products[i] <== values[i] * indicators[i];
        partialSums[i] <== partialSums[i - 1] + products[i];
    }
    
    out <== partialSums[N - 1];
}

// ============================================
// INCREMENT AT INDEX (with range enforcement)
// ============================================
template IncrementAt(N, MAX_VALUE) {
    signal input values[N];
    signal input index;
    signal output newValues[N];
    
    component indexBound = ZKLessThan(8);
    indexBound.in[0] <== index;
    indexBound.in[1] <== N;
    indexBound.out === 1;
    
    component isEq[N];
    component rangeBound[N];
    
    for (var i = 0; i < N; i++) {
        rangeBound[i] = Num2Bits(32);
        rangeBound[i].in <== values[i];
        
        isEq[i] = ZKIsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        
        newValues[i] <== values[i] + isEq[i].out;
    }
}

// ============================================
// CONDITIONAL SELECT
// ============================================
template ConditionalSelect() {
    signal input condition;
    signal input ifTrue;
    signal input ifFalse;
    signal output result;
    
    condition * (condition - 1) === 0;
    result <== condition * ifTrue + (1 - condition) * ifFalse;
}