pragma circom 2.1.0;

include "./comparators.circom";

template Selector(N) {
    signal input values[N];
    signal input index;
    signal output out;
    component isEq[N];
    signal indicators[N];
    for (var i = 0; i < N; i++) {
        isEq[i] = IsEqual();
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
    signal partialSums[N];
    products[0] <== values[0] * indicators[0];
    partialSums[0] <== products[0];
    for (var i = 1; i < N; i++) {
        products[i] <== values[i] * indicators[i];
        partialSums[i] <== partialSums[i - 1] + products[i];
    }
    out <== partialSums[N - 1];
}

template IncrementAt(N) {
    signal input values[N];
    signal input index;
    signal output newValues[N];
    component isEq[N];
    for (var i = 0; i < N; i++) {
        isEq[i] = IsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        newValues[i] <== values[i] + isEq[i].out;
    }
}

template SetAt(N) {
    signal input values[N];
    signal input index;
    signal input newValue;
    signal output newValues[N];
    component isEq[N];
    for (var i = 0; i < N; i++) {
        isEq[i] = IsEqual();
        isEq[i].in[0] <== index;
        isEq[i].in[1] <== i;
        newValues[i] <== values[i] * (1 - isEq[i].out) + newValue * isEq[i].out;
    }
}
