pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../../node_modules/circomlib/circuits/bitify.circom";
include "../lib/comparators.circom";

/**
 * Emergency Authentication
 * 
 * Security:
 * - Emergency key verified via EdDSA
 * - At least one emergency condition must be true
 * - TVL range-checked to prevent overflow before multiplication
 * - No hard constraint - parent enforces
 */
template EmergencyAuth() {
    // Public
    signal input messageHash;
    signal input expectedEmergencyKeyHash;
    signal input currentTVL;
    signal input normalTVL;
    signal input timeSinceLastBlock;
    signal input systemPaused;

    // Private
    signal input emergencyKeyHash;
    signal input emergencyPublicKeyX;
    signal input emergencyPublicKeyY;
    signal input emergencyR8x;
    signal input emergencyR8y;
    signal input emergencyS;

    signal output valid;

    // Step 1: Key hash match
    component keyMatch = ZKIsEqual();
    keyMatch.in[0] <== emergencyKeyHash;
    keyMatch.in[1] <== expectedEmergencyKeyHash;
    keyMatch.out === 1;

    // Step 2: EdDSA signature
    component sigVerifier = EdDSAMiMCVerifier();
    sigVerifier.enabled <== 1;
    sigVerifier.Ax <== emergencyPublicKeyX;
    sigVerifier.Ay <== emergencyPublicKeyY;
    sigVerifier.R8x <== emergencyR8x;
    sigVerifier.R8y <== emergencyR8y;
    sigVerifier.S <== emergencyS;
    sigVerifier.M <== messageHash;

    // Step 3: TVL range check before multiplication (prevent overflow)
    // normalTVL must fit in 63 bits (so *2 fits in 64 bits)
    component normalTVLBits = Num2Bits(63);
    normalTVLBits.in <== normalTVL;  // Fails if >= 2^63

    signal normalTVLThreshold;
    normalTVLThreshold <== normalTVL * 2;  // Safe: normalTVL < 2^63

    // Condition 1: TVL spike (current > normal * 2)
    component tvlSpike = ZKGreaterThan(64);
    tvlSpike.in[0] <== currentTVL;
    tvlSpike.in[1] <== normalTVLThreshold;

    // Condition 2: Chain halted (> 1 hour since last block)
    component chainHalted = ZKGreaterThan(32);
    chainHalted.in[0] <== timeSinceLastBlock;
    chainHalted.in[1] <== 3600;  // EMERGENCY_MAX_BLOCK_TIME

    // Condition 3: System paused
    component isPaused = ZKIsEqual();
    isPaused.in[0] <== systemPaused;
    isPaused.in[1] <== 1;

    // At least one condition true
    signal conditionSum;
    conditionSum <== tvlSpike.out + chainHalted.out + isPaused.out;

    component anyCondition = ZKGreaterThan(8);
    anyCondition.in[0] <== conditionSum;
    anyCondition.in[1] <== 0;
    anyCondition.out === 1;

    valid <== 1;
}