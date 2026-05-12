pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsaposeidon.circom";
include "../lib/comparators.circom";
include "../lib/validators.circom";

template AdminAuth(MAX_SIGNERS) {
    signal input messageHash;
    signal input requiredThreshold;
    signal input publicKeys[MAX_SIGNERS][2];
    signal input signatures[MAX_SIGNERS][3];
    signal input signerMask[MAX_SIGNERS];
    
    signal output valid;
    
    component maskValidators[MAX_SIGNERS];
    for (var i = 0; i < MAX_SIGNERS; i++) {
        maskValidators[i] = IsBinary();
        maskValidators[i].value <== signerMask[i];
        maskValidators[i].valid === 1;
    }
    
    component verifiers[MAX_SIGNERS];
    signal validCounts[MAX_SIGNERS + 1];
    validCounts[0] <== 0;
    
    for (var i = 0; i < MAX_SIGNERS; i++) {
        verifiers[i].valid === signerMask[i]; // enforce signature matches mask
        verifiers[i] = EdDSAPoseidonVerifier();
        verifiers[i].enabled <== signerMask[i];
        verifiers[i].Ax <== publicKeys[i][0];
        verifiers[i].Ay <== publicKeys[i][1];
        verifiers[i].R8x <== signatures[i][0];
        verifiers[i].R8y <== signatures[i][1];
        verifiers[i].S <== signatures[i][2];
        verifiers[i].M <== messageHash;
        
        validCounts[i + 1] <== validCounts[i] + signerMask[i];
    }
    
    component thresholdCheck = ZKGreaterEqThan(8);
    thresholdCheck.in[0] <== validCounts[MAX_SIGNERS];
    thresholdCheck.in[1] <== requiredThreshold;
    
    valid <== thresholdCheck.out;
}

