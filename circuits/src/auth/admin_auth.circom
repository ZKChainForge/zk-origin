pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/validators.circom";
include "../lib/constants.circom";

/*
 * Admin Authentication: M-of-N Multisig Verification
 * 
 * SECURITY FIX: Validates signer mask is binary (0 or 1)
 */

template AdminAuth(MAX_SIGNERS) {
    signal input messageHash;
    signal input requiredThreshold;
    
    signal input publicKeys[MAX_SIGNERS][2];
    signal input signatures[MAX_SIGNERS][2];
    signal input signerMask[MAX_SIGNERS];
    
    signal output valid;
    
    // ============ VALIDATE SIGNER MASKS ARE BINARY ============
    component maskValidators[MAX_SIGNERS];
    for (var i = 0; i < MAX_SIGNERS; i++) {
        maskValidators[i] = IsBinary();
        maskValidators[i].value <== signerMask[i];
        maskValidators[i].valid === 1;
    }
    
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
        
        validCounts[i + 1] <== validCounts[i] + 
            verifiers[i].valid * signerMask[i];
    }
    
    // ============ VERIFY THRESHOLD ============
    component thresholdCheck = ZKGreaterEqThan(8);
    thresholdCheck.in[0] <== validCounts[MAX_SIGNERS];
    thresholdCheck.in[1] <== requiredThreshold;
    thresholdCheck.out === 1;
    
    valid <== 1;
}

component main {public [messageHash, requiredThreshold]} = AdminAuth(15);