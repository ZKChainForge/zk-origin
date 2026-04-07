pragma circom 2.1.0;

/*
 * Emergency Authentication: Emergency Key and Conditions Verification
 * 
 * Verifies that:
 * 1. Emergency key is the authorized key
 * 2. Emergency conditions are met
 * 3. Emergency signature is valid
 */

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/poseidon.circom";

template EmergencyAuth() {
    // ============ PUBLIC INPUTS ============
    signal input messageHash;
    signal input expectedEmergencyKeyHash;
    signal input emergencyConditionsMet;
    
    // ============ PRIVATE INPUTS ============
    signal input emergencyKeyHash;
    signal input emergencySignatureR;
    signal input emergencySignatureS;
    signal input emergencyPublicKeyX;
    signal input emergencyPublicKeyY;
    
    // ============ OUTPUT ============
    signal output valid;
    
    // ============ VERIFY EMERGENCY KEY ============
    component keyMatch = IsEqual();
    keyMatch.in[0] <== emergencyKeyHash;
    keyMatch.in[1] <== expectedEmergencyKeyHash;
    keyMatch.out === 1;
    
    // ============ VERIFY EMERGENCY CONDITIONS ============
    component conditionCheck = IsEqual();
    conditionCheck.in[0] <== emergencyConditionsMet;
    conditionCheck.in[1] <== 1;
    conditionCheck.out === 1;
    
    // ============ VERIFY EMERGENCY SIGNATURE ============
    component sigVerifier = EdDSAVerifier();
    sigVerifier.M <== messageHash;
    sigVerifier.Ax <== emergencyPublicKeyX;
    sigVerifier.Ay <== emergencyPublicKeyY;
    sigVerifier.R8x <== emergencySignatureR;
    sigVerifier.R8y <== emergencySignatureS;
    sigVerifier.valid === 1;
    
    valid <== 1;
}

component main {public [
    messageHash,
    expectedEmergencyKeyHash,
    emergencyConditionsMet
]} = EmergencyAuth();