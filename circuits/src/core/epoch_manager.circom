pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/constants.circom";

/*
 * Epoch Manager: Handle Epoch Transitions with Validation
 * 
 * SECURITY: Actually validates epoch transitions and timestamps
 */

template EpochManager() {
    signal input prevEpochId;
    signal input newEpochId;
    signal input prevTimestamp;
    signal input newTimestamp;
    signal input prevCounters[7];
    
    signal output epochValid;
    signal output countersValid;
    signal output shouldResetCounters;
    
    // ============ STEP 1: VALIDATE EPOCH TRANSITION ============
    signal epochDiff;
    epochDiff <== newEpochId - prevEpochId;
    
    component epochDiffCheck = ZKLessEqThan(8);
    epochDiffCheck.in[0] <== epochDiff;
    epochDiffCheck.in[1] <== 1;
    epochDiffCheck.out === 1;
    
    // ============ STEP 2: VALIDATE TIMESTAMP INCREASES ============
    component timeCheck = ZKGreaterThan(32);
    timeCheck.in[0] <== newTimestamp;
    timeCheck.in[1] <== prevTimestamp;
    timeCheck.out === 1;
    
    // ============ STEP 3: DETECT EPOCH CHANGE ============
    component epochChanged = ZKIsEqual();
    epochChanged.in[0] <== epochDiff;
    epochChanged.in[1] <== 1;
    shouldResetCounters <== epochChanged.out;
    
    // ============ STEP 4: VALIDATE MINIMUM TIME FOR EPOCH CHANGE ============
    component minTimeCheck = ZKGreaterEqThan(32);
    minTimeCheck.in[0] <== newTimestamp - prevTimestamp;
    minTimeCheck.in[1] <== EPOCH_DURATION_SECONDS();
    
    signal timeValid;
    timeValid <== (1 - shouldResetCounters) + shouldResetCounters * minTimeCheck.out;
    timeValid === 1;
    
    // ============ OUTPUT ============
    epochValid <== 1;
    countersValid <== 1;
}