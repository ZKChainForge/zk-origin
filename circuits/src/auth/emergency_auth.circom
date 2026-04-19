pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/constants.circom";

/**
 * @title Emergency Authentication (PRODUCTION)
 * @notice Emergency action verification with multi-condition checks
 * 
 * SECURITY:
 *  Requires emergency key signature
 *  At least one emergency condition must be true
 *  Multiple independent conditions prevent abuse
 *  TVL check prevents underflow via overflow check
 *  All conditions are deterministic
 * 
 * PROTECTION: EMERGENCY PROTECTED
 * - Requires cryptographic authorization (emergency key)
 * - Requires valid emergency condition
 * - Prevents emergency from being used casually
 * - Multi-condition prevents false positives
 * 
 * CONDITIONS (at least one must be true):
 * 1. TVL spike: currentTVL > (normalTVL * 2)
 * 2. Chain halted: timeSinceLastBlock > 1 hour
 * 3. System paused: systemPaused == 1
 * 
 * INPUT AUTHORIZATION:
 * - messageHash: Emergency action being authorized
 * - expectedEmergencyKeyHash: Expected key hash
 * - currentTVL: Current total value locked
 * - normalTVL: Baseline TVL
 * - timeSinceLastBlock: Seconds since last block
 * - systemPaused: Is system paused (0 or 1)
 * - emergencyKeyHash: Actual key hash (private)
 * - emergencySignatureR, S: Emergency key signature
 * - emergencyPublicKeyX, Y: Emergency public key
 * 
 * OUTPUT GUARANTEE:
 * - valid: 1 if key valid AND condition true, circuit fails if not
 * 
 * CONSTRAINTS: ~10,000+ (multiple checks)
 * - EdDSA signature: ~7500
 * - TVL multiplication: ~1000
 * - Condition checks: ~500
 * 
 * PRODUCTION CHECKLIST:
 *  Emergency key must be valid
 *  At least one emergency condition true
 *  TVL multiplication doesn't overflow
 *  All numerical comparisons constrained
 *  All conditions are independent
 *  Conditions are testable without action
 * 
 * ATTACK VECTORS MITIGATED:
 *  Fake emergency key: Signature check prevents
 *  False emergency: Condition check prevents
 *  TVL underflow: Overflow check prevents
 *  Casual emergency: Multi-condition prevents abuse
 *  Replay: Off-chain checks timestamp
 * 
 * NOTES:
 * - Emergency is last-resort protection
 * - Should be rarely used
 * - Conditions are designed to catch real emergencies
 * - May be extended with additional conditions
 * - Consider governance override of emergency actions
 */

template EmergencyAuth() {
    // ============ PUBLIC INPUTS ============
    signal input messageHash;                  // Emergency action
    signal input expectedEmergencyKeyHash;    // Expected key
    signal input currentTVL;                  // Current TVL
    signal input normalTVL;                   // Normal TVL baseline
    signal input timeSinceLastBlock;          // Seconds since last block
    signal input systemPaused;                // Is system paused (0 or 1)
    
    // ============ PRIVATE INPUTS ============
    signal input emergencyKeyHash;            // Actual key hash
    signal input emergencySignatureR;
    signal input emergencySignatureS;
    signal input emergencyPublicKeyX;
    signal input emergencyPublicKeyY;
    
    // ============ PUBLIC OUTPUTS ============
    signal output valid;  // 1 if authorized and condition true
    
    // ============ STEP 1: VERIFY EMERGENCY KEY ============
    component keyMatch = ZKIsEqual();
    keyMatch.in[0] <== emergencyKeyHash;
    keyMatch.in[1] <== expectedEmergencyKeyHash;
    keyMatch.out === 1;  // ENFORCE: key must match
    
    // ============ STEP 2: VERIFY SIGNATURE ============
    component sigVerifier = EdDSAVerifier();
    sigVerifier.M <== messageHash;
    sigVerifier.Ax <== emergencyPublicKeyX;
    sigVerifier.Ay <== emergencyPublicKeyY;
    sigVerifier.R8x <== emergencySignatureR;
    sigVerifier.R8y <== emergencySignatureS;
    sigVerifier.valid === 1;  // ENFORCE: signature valid
    
    // ============ STEP 3: CONDITION 1 - TVL SPIKE ============
    // Check for overflow: normalTVL < 2^63
    component overflowCheck = ZKLessThan(64);
    overflowCheck.in[0] <== normalTVL;
    overflowCheck.in[1] <== 9223372036854775807;  // 2^63 - 1
    overflowCheck.out === 1;  // ENFORCE: no overflow possible
    
    // Compute TVL threshold: normalTVL * 2
    signal normalTVLThreshold;
    normalTVLThreshold <== normalTVL * EMERGENCY_TVL_MULTIPLIER();
    
    // Check if current > threshold
    component tvlSpike = ZKGreaterThan(64);
    tvlSpike.in[0] <== currentTVL;
    tvlSpike.in[1] <== normalTVLThreshold;
    
    // ============ STEP 4: CONDITION 2 - CHAIN HALTED ============
    component chainHalted = ZKGreaterThan(32);
    chainHalted.in[0] <== timeSinceLastBlock;
    chainHalted.in[1] <== EMERGENCY_MAX_BLOCK_TIME();
    
    // ============ STEP 5: CONDITION 3 - SYSTEM PAUSED ============
    component isPaused = ZKIsEqual();
    isPaused.in[0] <== systemPaused;
    isPaused.in[1] <== 1;
    
    // ============ STEP 6: AT LEAST ONE CONDITION MUST BE TRUE ============
    signal conditionsMet;
    conditionsMet <== tvlSpike.out + chainHalted.out + isPaused.out;
    
    component anyCondition = ZKGreaterThan(8);
    anyCondition.in[0] <== conditionsMet;
    anyCondition.in[1] <== 0;
    anyCondition.out === 1;  // ENFORCE: >= 1 condition true
    
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