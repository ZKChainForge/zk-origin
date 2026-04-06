pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";

// Admin Authentication: Verify M-of-N multisig
template AdminAuthCircuit(MAX_SIGNERS) {
    signal input messageHash;
    signal input signers[MAX_SIGNERS][2];          // (X, Y) public keys
    signal input signatures[MAX_SIGNERS][2];       // (R, S) signature components
    signal input signerMask[MAX_SIGNERS];          // 1 if signer participated
    signal input requiredThreshold;
    signal output valid;
    
    // Verify each signature
    component verifiers[MAX_SIGNERS];
    signal validCounts[MAX_SIGNERS + 1];
    validCounts[0] <== 0;
    
    for (var i = 0; i < MAX_SIGNERS; i++) {
        verifiers[i] = EdDSAVerifier();
        verifiers[i].M <== messageHash;
        verifiers[i].Ax <== signers[i][0];
        verifiers[i].Ay <== signers[i][1];
        verifiers[i].R8x <== signatures[i][0];
        verifiers[i].R8y <== signatures[i][1];
        verifiers[i].enabled <== signerMask[i];
        
        // Count valid signatures (only if mask is 1)
        validCounts[i + 1] <== validCounts[i] + 
            verifiers[i].valid * signerMask[i];
    }
    
    // Verify threshold met
    component thresholdCheck = GreaterEqThan(8);
    thresholdCheck.in[0] <== validCounts[MAX_SIGNERS];
    thresholdCheck.in[1] <== requiredThreshold;
    thresholdCheck.out === 1;
    
    valid <== 1;
}

component main {public [messageHash]} = AdminAuthCircuit(5);