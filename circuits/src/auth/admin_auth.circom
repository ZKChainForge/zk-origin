pragma circom 2.1.0;

/*
 * Admin Authentication: M-of-N Multisig Verification
 * 
 * Verifies that a state transition has at least THRESHOLD valid signatures
 * from the authorized admin signers.
 */

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/constants.circom";

template AdminAuth(MAX_SIGNERS) {
    // ============ PUBLIC INPUTS ============
    signal input messageHash;
    signal input requiredThreshold;
    
    // ============ PRIVATE INPUTS ============
    signal input publicKeys[MAX_SIGNERS][2];      // (X, Y) coordinates
    signal input signatures[MAX_SIGNERS][2];      // (R, S) components
    signal input signerMask[MAX_SIGNERS];         // 1 if signer participated
    
    // ============ OUTPUT ============
    signal output valid;
    
    // ============ VERIFY EACH SIGNATURE ============
    component verifiers[MAX_SIGNERS];
    signal validCounts[MAX_SIGNERS + 1];
    validCounts[0] <== 0;
    
    for (var i = 0; i < MAX_SIGNERS; i++) {
        verifiers[i] = EdDSAVerifier();
        verifiers[i].M <== messageHash;
        verifiers[i].Ax <== publicKeys[i][0];
        verifiers[i].Ay <== publicKeys[i][1];
        verifiers[i].R8x <== signatures[i][0];
        verifiers[i].R8y <== signatures[i][1];
        
        // Count valid signatures where signer mask is 1
        validCounts[i + 1] <== validCounts[i] + 
            verifiers[i].valid * signerMask[i];
    }
    
    // ============ VERIFY THRESHOLD ============
    component thresholdCheck = GreaterEqThan(8);
    thresholdCheck.in[0] <== validCounts[MAX_SIGNERS];
    thresholdCheck.in[1] <== requiredThreshold;
    
    // Must meet or exceed threshold
    thresholdCheck.out === 1;
    
    valid <== 1;
}

component main {public [messageHash, requiredThreshold]} = AdminAuth(15);