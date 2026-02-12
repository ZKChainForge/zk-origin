pragma circom 2.1.0;

//Selector/Multiplexer circuits for ZK-ORIGIN
//Used for selecting values based on origin class


include "./comparators.circom";

// Select one value from an array based on index
template Selector(N) {
    signal input values[N];
    signal input index;
    signal output out;
    
    // Create indicator variables
    component isEq[N];
    signal indicators[N];
    
    for (var i = 0; i < N; i++) {
        isEq[i] = IsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        indicators[i] <== isEq[i].out;
    }
    
    // Sum of indicators must be 1 (exactly one selected)
    signal indicatorSum[N];
    indicatorSum[0] <== indicators[0];
    for (var i = 1; i < N; i++) {
        indicatorSum[i] <== indicatorSum[i-1] + indicators[i];
    }
    indicatorSum[N-1] === 1;
    
    // Compute weighted sum
    signal products[N];
    signal partialSums[N];
    
    products[0] <== values[0] * indicators[0];
    partialSums[0] <== products[0];
    
    for (var i = 1; i < N; i++) {
        products[i] <== values[i] * indicators[i];
        partialSums[i] <== partialSums[i-1] + products[i];
    }
    
    out <== partialSums[N-1];
}

// Increment one value in an array based on index
template IncrementAt(N) {
    signal input values[N];
    signal input index;
    signal output newValues[N];
    
    component isEq[N];
    
    for (var i = 0; i < N; i++) {
        isEq[i] = IsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        
        // Increment by 1 if this is the selected index
        newValues[i] <== values[i] + isEq[i].out;
    }
}