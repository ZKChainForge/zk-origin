pragma circom 2.1.0;

include "../lib/comparators.circom";

/**
 * System Authentication
 * Verifies caller is the authorized system address
 * No hard constraint - parent enforces
 */
template SystemAuth() {
    signal input callerAddress;
    signal input expectedSystemAddress;

    signal output valid;

    component addrMatch = ZKIsEqual();
    addrMatch.in[0] <== callerAddress;
    addrMatch.in[1] <== expectedSystemAddress;

    valid <== addrMatch.out;
    // Parent MUST enforce: selectedAuth === 1
}