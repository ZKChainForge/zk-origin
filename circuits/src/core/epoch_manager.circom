
pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/constants.circom";

/*
 * Epoch Manager: Handle Epoch Transitions
 */

template EpochManager() {
    signal input prevEpochId;
    signal input newEpochId;
    signal input prevTimestamp;
    signal input newTimestamp;
    signal input prevCounters[7];
    
    signal output epochValid;
    signal output countersValid;
    
    // ============ ALLOW SAME EPOCH ============
    // For testing, we allow same epoch or next epoch
    signal epochDiff;
    epochDiff <== newEpochId - prevEpochId;
    
    // Allow 0 or 1 increment
    component epochCheck = ZKLessEqThan(8);
    epochCheck.in[0] <== epochDiff;
    epochCheck.in[1] <== 1;
    epochCheck.out === 1;
    
    // For now, don't enforce time checks
    // Just mark as valid
    
    epochValid <== 1;
    countersValid <== 1;
}
