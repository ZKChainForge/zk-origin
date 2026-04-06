pragma circom 2.1.0;

include "../lib/comparators.circom";

template BridgeAuthCircuit() {
    signal input sourceChainId;
    signal input expectedSourceChain;
    signal output valid;

    component chainMatch = IsEqual();
    chainMatch.in[0] <== sourceChainId;
    chainMatch.in[1] <== expectedSourceChain;

    valid <== chainMatch.out;
}

component main {public [sourceChainId, expectedSourceChain]} = BridgeAuthCircuit();
