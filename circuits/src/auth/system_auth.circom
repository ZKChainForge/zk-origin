pragma circom 2.1.0;

include "../lib/comparators.circom";

template SystemAuthCircuit() {
    signal input callerAddress;
    signal input authorizedAddress;
    signal output valid;

    component addrMatch = IsEqual();
    addrMatch.in[0] <== callerAddress;
    addrMatch.in[1] <== authorizedAddress;

    valid <== addrMatch.out;
}

component main {public [callerAddress, authorizedAddress]} = SystemAuthCircuit();
