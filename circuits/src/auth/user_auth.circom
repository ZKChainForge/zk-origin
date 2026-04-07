pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";

/*
 * User Authentication: EdDSA Signature Verification
 * 
 * Verifies that a state transition is signed by the user.
 * Uses EdDSA (Ed25519) from circomlib.
 */

template UserAuth() {
    // ============ PUBLIC INPUTS ============
    signal input messageHash;
    
    // ============ PRIVATE INPUTS ============
    signal input publicKeyX;
    signal input publicKeyY;
    signal input signatureR;
    signal input signatureS;
    
    // ============ OUTPUT ============
    signal output valid;
    
    // ============ VERIFY EdDSA SIGNATURE ============
    component sigVerifier = EdDSAVerifier();
    sigVerifier.M <== messageHash;
    sigVerifier.Ax <== publicKeyX;
    sigVerifier.Ay <== publicKeyY;
    sigVerifier.R8x <== signatureR;
    sigVerifier.R8y <== signatureS;
    
    // Signature MUST be valid
    sigVerifier.valid === 1;
    
    valid <== 1;
}