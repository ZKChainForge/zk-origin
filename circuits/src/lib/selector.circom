pragma circom 2.1.0;

include "./comparators.circom";

/*
 * Array Selectors and Updaters
 * 
 * Conditional selection and updates for arrays.
 * Efficient constraint implementations.
 */

// ============================================
// SELECTOR: Pick value at index
// ============================================
template Selector(N) {
    signal input values[N];
    signal input index;
    signal output out;
    
    // Declare all components and signals first
    component isEq[N];
    signal indicators[N];
    signal indicatorSum[N];
    signal products[N];
    signal partialSums[N];
    
    // Create indicators for each position
    for (var i = 0; i < N; i++) {
        isEq[i] = ZKIsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        indicators[i] <== isEq[i].out;
    }
    
    // Verify exactly one indicator is 1
    indicatorSum[0] <== indicators[0];
    for (var i = 1; i < N; i++) {
        indicatorSum[i] <== indicatorSum[i - 1] + indicators[i];
    }
    indicatorSum[N - 1] === 1;
    
    // Compute weighted sum
    products[0] <== values[0] * indicators[0];
    partialSums[0] <== products[0];
    for (var i = 1; i < N; i++) {
        products[i] <== values[i] * indicators[i];
        partialSums[i] <== partialSums[i - 1] + products[i];
    }
    
    out <== partialSums[N - 1];
}

// ============================================
// INCREMENT AT INDEX
// ============================================
template IncrementAt(N) {
    signal input values[N];
    signal input index;
    signal output newValues[N];
    
    // Declare components first
    component isEq[N];
    
    for (var i = 0; i < N; i++) {
        isEq[i] = ZKIsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        newValues[i] <== values[i] + isEq[i].out;
    }
}

// ============================================
// SET AT INDEX
// ============================================
template SetAt(N) {
    signal input values[N];
    signal input index;
    signal input newValue;
    signal output newValues[N];
    
    // Declare components first
    component isEq[N];
    
    for (var i = 0; i < N; i++) {
        isEq[i] = ZKIsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        newValues[i] <== values[i] * (1 - isEq[i].out) + newValue * isEq[i].out;
    }
}

// ============================================
// ARRAY SUM
// ============================================
template ArraySum(N) {
    signal input values[N];
    signal output sum;
    
    // Declare all signals first
    signal partialSums[N];
    
    partialSums[0] <== values[0];
    for (var i = 1; i < N; i++) {
        partialSums[i] <== partialSums[i - 1] + values[i];
    }
    
    sum <== partialSums[N - 1];
}