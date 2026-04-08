
pragma circom 2.1.0;

include "./comparators.circom";

/*
 * Array Selectors and Updaters
 */

// ============ SELECTOR: Pick value at index ============
template Selector(N) {
    signal input values[N];
    signal input index;
    signal output out;
    
    component isEq[N];
    signal indicators[N];
    signal indicatorSum[N];
    signal products[N];
    signal partialSums[N];
    
    for (var i = 0; i < N; i++) {
        isEq[i] = ZKIsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        indicators[i] <== isEq[i].out;
    }
    
    indicatorSum[0] <== indicators[0];
    for (var i = 1; i < N; i++) {
        indicatorSum[i] <== indicatorSum[i - 1] + indicators[i];
    }
    indicatorSum[N - 1] === 1;
    
    products[0] <== values[0] * indicators[0];
    partialSums[0] <== products[0];
    for (var i = 1; i < N; i++) {
        products[i] <== values[i] * indicators[i];
        partialSums[i] <== partialSums[i - 1] + products[i];
    }
    
    out <== partialSums[N - 1];
}

// ============ INCREMENT AT INDEX (WITH OVERFLOW CHECK) ============
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
        
        overflowCheck[i] = ZKLessThan(32);
        overflowCheck[i].in[0] <== values[i];
        overflowCheck[i].in[1] <== MAX_VALUE;
        
        overflowOk[i] <== (1 - isEq[i].out) + isEq[i].out * overflowCheck[i].out;
        overflowOk[i] === 1;
        
        newValues[i] <== values[i] + isEq[i].out;
    }
}

// ============ SET AT INDEX ============
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

// ============ ARRAY SUM ============
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
