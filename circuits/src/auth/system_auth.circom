pragma circom 2.1.0;

/*
 * System Authentication: Authorized System Caller Verification
 * 
 * Verifies that a system-level operation is called from an authorized address.
 */

include "../lib/comparators.circom";

template SystemAuth() {
    // ============ PUBLIC INPUTS ============
    signal input callerAddress;
    signal input expectedSystemAddress;
    
    // ============ OUTPUT ============
    signal output valid;
    
    // ============ VERIFY ADDRESS MATCHES ============
    component addrMatch = IsEqual();
    addrMatch.in[0] <== callerAddress;
    addrMatch.in[1] <== expectedSystemAddress;
    addrMatch.out === 1;
    
    valid <== 1;
}

component main {public [callerAddress, expectedSystemAddress]} = SystemAuth();