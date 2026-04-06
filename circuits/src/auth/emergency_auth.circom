pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/poseidon.circom";

// Emergency Authentication: Verify emergency key and conditions
template EmergencyAuthCircuit() {
    signal input emergencyKeyHash;
    signal input expectedEmergencyKeyHash;
    signal input emergencyConditionsMet;
    signal input emergencySignature[2];            // (R, S)
    signal input emergencyPublicKey[2];            // (X, Y)
    signal input messageHash;
    signal output valid;
    
    // 1. Verify emergency key hash
    component keyMatch = IsEqual();
    keyMatch.in[0] <== emergencyKeyHash;
    keyMatch.in[1] <== expectedEmergencyKeyHash;
    keyMatch.out === 1;
    
    // 2. Verify emergency conditions are met
    component conditionCheck = IsEqual();
    conditionCheck.in[0] <== emergencyConditionsMet;
    conditionCheck.in[1] <== 1;
    conditionCheck.out === 1;
    
    // 3. Verify emergency signature
    component sigVerifier = EdDSAVerifier();
    sigVerifier.M <== messageHash;
    sigVerifier.Ax <== emergencyPublicKey[0];
    sigVerifier.Ay <== emergencyPublicKey[1];
    sigVerifier.R8x <== emergencySignature[0];
    sigVerifier.R8y <== emergencySignature[1];
    sigVerifier.enabled <== 1;
    sigVerifier.valid === 1;
    
    valid <== 1;
}

component main {public [messageHash]} = EmergencyAuthCircuit();