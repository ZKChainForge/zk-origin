pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/poseidon.circom";
include "../lib/comparators.circom";

// User Authentication: Verify EdDSA signature
template UserAuthCircuit() {
    signal input messageHash;
    signal input publicKeyX;
    signal input publicKeyY;
    signal input signatureR;
    signal input signatureS;
    signal output valid;
    
    // Verify EdDSA signature
    component sigVerifier = EdDSAVerifier();
    sigVerifier.M <== messageHash;
    sigVerifier.Ax <== publicKeyX;
    sigVerifier.Ay <== publicKeyY;
    sigVerifier.R8x <== signatureR;
    sigVerifier.R8y <== signatureS;
    sigVerifier.enabled <== 1;
    
    // Output must be 1 (signature valid)
    valid <== sigVerifier.valid;
    sigVerifier.valid === 1;
}

component main {public [messageHash]} = UserAuthCircuit();