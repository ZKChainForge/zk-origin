pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/constants.circom";

template EpochManager() {
    signal input prevEpochId;
    signal input newEpochId;
    signal input prevTimestamp;
    signal input newTimestamp;
    signal input prevCounters[7];
    
    signal output epochValid;
    signal output countersValid;
    signal output shouldResetCounters;
    
    signal epochDiff;
    epochDiff <== newEpochId - prevEpochId;
    
    component epochDiffCheck = ZKLessEqThan(8);
    epochDiffCheck.in[0] <== epochDiff;
    epochDiffCheck.in[1] <== 1;
    epochDiffCheck.out === 1;
    
    component timeCheck = ZKGreaterThan(32);
    timeCheck.in[0] <== newTimestamp;
    timeCheck.in[1] <== prevTimestamp;
    timeCheck.out === 1;
    
    component epochChanged = ZKIsEqual();
    epochChanged.in[0] <== epochDiff;
    epochChanged.in[1] <== 1;
    shouldResetCounters <== epochChanged.out;
    
    component minTimeCheck = ZKGreaterEqThan(32);
    minTimeCheck.in[0] <== newTimestamp - prevTimestamp;
    minTimeCheck.in[1] <== EPOCH_DURATION_SECONDS();
    
    signal timeValid;
    timeValid <== (1 - shouldResetCounters) + shouldResetCounters * minTimeCheck.out;
    timeValid === 1;
    
    epochValid <== 1;
    countersValid <== 1;
}