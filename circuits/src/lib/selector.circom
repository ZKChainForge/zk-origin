pragma circom 2.1.0;

include "./comparators.circom";

/**
 * @title Array Selection and Update Operations (PRODUCTION)
 * @notice Deterministic array indexing without branches
 * 
 * SECURITY:
 *  All selections use constraint equations
 *  No conditional branching
 *  Index validation mandatory
 * 
 * PRODUCTION NOTES:
 * - Selector uses one-hot encoding for safety
 * - IncrementAt includes overflow checks
 * - All indices must be valid (0 <= index < N)
 * 
 * CONSTRAINTS:
 * - Selector(N): ~N constraints
 * - IncrementAt(N): ~3*N constraints
 * - SetAt(N): ~2*N constraints
 * - ArraySum(N): ~N constraints
 */

// ============================================
// SELECTOR: Pick value at index
// ============================================
template Selector(N) {
    signal input values[N];
    signal input index;
    signal output out;
    
    component isEq[N];
    signal indicators[N];
    signal indicatorSum[N];
    signal products[N];
    signal partialSums[N];
    
    // Create one-hot encoding
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
    indicatorSum[N - 1] === 1;  // ENFORCE: exactly one match
    
    // Compute selected value
    products[0] <== values[0] * indicators[0];
    partialSums[0] <== products[0];
    for (var i = 1; i < N; i++) {
        products[i] <== values[i] * indicators[i];
        partialSums[i] <== partialSums[i - 1] + products[i];
    }
    
    out <== partialSums[N - 1];
}

// ============================================
// INCREMENT AT INDEX (with overflow protection)
// ============================================
template IncrementAt(N, MAX_VALUE) {
    signal input values[N];
    signal input index;
    signal output newValues[N];
    
    component isEq[N];
    component overflowCheck[N];
    signal overflowOk[N];
    
    for (var i = 0; i < N; i++) {
        isEq[i] = ZKIsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        
        // Check if incrementing would overflow
        overflowCheck[i] = ZKLessThan(32);
        overflowCheck[i].in[0] <== values[i];
        overflowCheck[i].in[1] <== MAX_VALUE;
        
        // If this is the selected index, must not overflow
        // If not the selected index, always OK
        overflowOk[i] <== (1 - isEq[i].out) + isEq[i].out * overflowCheck[i].out;
        overflowOk[i] === 1;  // ENFORCE: no overflow
        
        // Increment if selected, otherwise keep same
        newValues[i] <== values[i] + isEq[i].out;
    }
}

// ============================================
// SET AT INDEX (replace value)
// ============================================
template SetAt(N) {
    signal input values[N];
    signal input index;
    signal input newValue;
    signal output newValues[N];
    
    component isEq[N];
    
    for (var i = 0; i < N; i++) {
        isEq[i] = ZKIsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        newValues[i] <== values[i] * (1 - isEq[i].out) + newValue * isEq[i].out;
    }
}

// ============================================
// ARRAY SUM (accumulate all elements)
// ============================================
template ArraySum(N) {
    signal input values[N];
    signal output sum;
    
    signal partialSums[N];
    
    partialSums[0] <== values[0];
    for (var i = 1; i < N; i++) {
        partialSums[i] <== partialSums[i - 1] + values[i];
    }
    
    sum <== partialSums[N - 1];
}

// ============================================
// ARRAY PRODUCT (multiply all elements)
// ============================================
template ArrayProduct(N) {
    signal input values[N];
    signal output product;
    
    signal partialProducts[N];
    
    partialProducts[0] <== values[0];
    for (var i = 1; i < N; i++) {
        partialProducts[i] <== partialProducts[i - 1] * values[i];
    }
    
    product <== partialProducts[N - 1];
}

// ============================================
// CONDITIONAL SELECT (if condition then a else b)
// ============================================
template ConditionalSelect() {
    signal input condition;  // Must be 0 or 1
    signal input ifTrue;
    signal input ifFalse;
    signal output result;
    
    // Verify condition is binary
    condition * (condition - 1) === 0;
    
    result <== condition * ifTrue + (1 - condition) * ifFalse;
}

// ============================================
// MULTI-WAY SELECT (advanced)
// ============================================
template MultiSelect(N) {
    signal input values[N];
    signal input index;
    signal output selected;
    
    component selector = Selector(N);
    for (var i = 0; i < N; i++) {
        selector.values[i] <== values[i];
    }
    selector.index <== index;
    selected <== selector.out;
}