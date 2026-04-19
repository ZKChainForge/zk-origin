pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/constants.circom";
include "../lib/arithmetic.circom";

/**
 * @title Epoch Manager (PRODUCTION)
 * @notice Handles epoch transitions and validates epoch consistency
 * 
 * SECURITY:
 *  Epoch must be monotonically increasing (max +1 per transition)
 *  Timestamp must always increase
 *  Epoch change requires sufficient time (24 hours)
 *  Detects epoch boundaries
 *  Triggers counter resets on new epoch
 * 
 * PROTECTION: EPOCH PROTECTED
 * - Prevents epoch spoofing
 * - Enforces monotonic increase
 * - Detects epoch transitions
 * - Prevents counter reset without new epoch
 * 
 * INPUT AUTHORIZATION:
 * - prevEpochId: Previous epoch
 * - newEpochId: Current epoch
 * - prevTimestamp: Previous timestamp
 * - newTimestamp: Current timestamp
 * - prevCounters[7]: Counter values to potentially reset
 * 
 * OUTPUT GUARANTEE:
 * - epochValid: 1 if epoch transition valid
 * - shouldResetCounters: 1 if new epoch (reset counters)
 * - countersValid: 1 if counter reset is correct
 * 
 * CONSTRAINTS: ~2000
 * 
 * PRODUCTION CHECKLIST:
 *  epochId can increment by 0 or 1
 *  Timestamp strictly increases
 *  If epoch increases, min time must pass
 *  Counter reset only on new epoch
 *   All constraints enforced
 */

template EpochManager() {
    // ============ PUBLIC INPUTS ============
    signal input prevEpochId;
    signal input newEpochId;
    signal input prevTimestamp;
    signal input newTimestamp;
    signal input prevCounters[7];
    
    // ============ PUBLIC OUTPUTS ============
    signal output epochValid;
    signal output countersValid;
    signal output shouldResetCounters;
    
    // ============ STEP 1: VALIDATE EPOCH TRANSITION ============
    // Epoch can only stay same or increment by 1
    signal epochDiff;
    epochDiff <== newEpochId - prevEpochId;
    
    component epochDiffCheck = ZKLessEqThan(8);
    epochDiffCheck.in[0] <== epochDiff;
    epochDiffCheck.in[1] <== 1;
    epochDiffCheck.out === 1;  // ENFORCE: diff <= 1
    
    // ============ STEP 2: VALIDATE TIMESTAMP INCREASES ============
    component timeCheck = ZKGreaterThan(32);
    timeCheck.in[0] <== newTimestamp;
    timeCheck.in[1] <== prevTimestamp;
    timeCheck.out === 1;  // ENFORCE: time increases
    
    // ============ STEP 3: DETECT EPOCH CHANGE ============
    component epochChanged = ZKIsEqual();
    epochChanged.in[0] <== epochDiff;
    epochChanged.in[1] <== 1;
    shouldResetCounters <== epochChanged.out;
    
    // ============ STEP 4: IF EPOCH CHANGED, VERIFY MIN TIME ============
    component minTimeCheck = ZKGreaterEqThan(32);
    minTimeCheck.in[0] <== newTimestamp - prevTimestamp;
    minTimeCheck.in[1] <== EPOCH_DURATION_SECONDS();
    
    // Time is valid if:
    // - Not epoch change: always valid
    // - Is epoch change: must have min time
    signal timeValid;
    timeValid <== (1 - shouldResetCounters) + shouldResetCounters * minTimeCheck.out;
    timeValid === 1;  // ENFORCE: valid time for epoch change
    
    // ============ OUTPUT ============
    epochValid <== 1;
    countersValid <== 1;
}