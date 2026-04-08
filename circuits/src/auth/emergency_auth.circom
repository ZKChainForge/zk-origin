pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/poseidon.circom";
include "../lib/constants.circom";

/*
 * Emergency Authentication: Emergency Key and Conditions Verification
 * 
 * SECURITY FIX: Actually verifies emergency conditions (not just trust prover)
 */

template EmergencyAuth() {
    signal input messageHash;
    signal input expectedEmergencyKeyHash;
    
    // ============ EMERGENCY CONDITIONS ============
    signal input currentTVL;
    signal input normalTVL;
    signal input timeSinceLastBlock;
    signal input systemPaused;
    
    // ============ PRIVATE INPUTS ============
    signal input emergencyKeyHash;
    signal input emergencySignatureR;
    signal input emergencySignatureS;
    signal input emergencyPublicKeyX;
    signal input emergencyPublicKeyY;
    
    signal output valid;
    
    // ============ VERIFY EMERGENCY KEY ============
    component keyMatch = ZKIsEqual();
    keyMatch.in[0] <== emergencyKeyHash;
    keyMatch.in[1] <== expectedEmergencyKeyHash;
    keyMatch.out === 1;
    
    // ============ CONDITION 1: TVL SPIKE ============
    signal normalTVLThreshold;
    normalTVLThreshold <== normalTVL * EMERGENCY_TVL_MULTIPLIER();
    
    component tvlSpike = ZKGreaterThan(64);
    tvlSpike.in[0] <== currentTVL;
    tvlSpike.in[1] <== normalTVLThreshold;
    
    // ============ CONDITION 2: CHAIN HALTED ============
    component chainHalted = ZKGreaterThan(32);
    chainHalted.in[0] <== timeSinceLastBlock;
    chainHalted.in[1] <== EMERGENCY_MAX_BLOCK_TIME();
    
    // ============ CONDITION 3: SYSTEM PAUSED ============
    component isPaused = ZKIsEqual();
    isPaused.in[0] <== systemPaused;
    isPaused.in[1] <== 1;
    
    // ============ AT LEAST ONE CONDITION MUST BE MET ============
    signal conditionsMet;
    conditionsMet <== tvlSpike.out + chainHalted.out + isPaused.out;
    
    component anyCondition = ZKGreaterThan(8);
    anyCondition.in[0] <== conditionsMet;
    anyCondition.in[1] <== 0;
    anyCondition.out === 1;
    
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
    currentTVL,
    normalTVL,
    timeSinceLastBlock,
    systemPaused
]} = EmergencyAuth();