pragma circom 2.1.0;

include "../lib/comparators.circom";

template AdminAuthCircuit() {
    signal input signatureCount;
    signal input requiredThreshold;
    signal output valid;

    component thresholdCheck = GreaterEqThan(8);
    thresholdCheck.in[0] <== signatureCount;
    thresholdCheck.in[1] <== requiredThreshold;

    valid <== thresholdCheck.out;
}

component main {public [signatureCount, requiredThreshold]} = AdminAuthCircuit();
