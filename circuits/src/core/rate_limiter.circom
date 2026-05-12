pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/selector.circom";
include "../lib/constants.circom";

template RateLimiter() {
    signal input epochId;
    signal input newOriginClass;
    signal input prevCounterCommitment;
    
    signal input prevCounters[7];
    signal input rateLimits[7];
    
    signal output rateLimitOk;
    signal output newCounterCommitment;
    
    component prevHasher = PoseidonHash8();
    prevHasher.in[0] <== epochId;
    for (var i = 0; i < 7; i++) {
        prevHasher.in[i + 1] <== prevCounters[i];
    }
    
    prevHasher.out === prevCounterCommitment;
    
    component counterSelector = Selector(7);
    for (var i = 0; i < 7; i++) {
        counterSelector.values[i] <== prevCounters[i];
    }
    counterSelector.index <== newOriginClass;
    signal currentCount;
    currentCount <== counterSelector.out;
    
    component limitSelector = Selector(7);
    for (var i = 0; i < 7; i++) {
        limitSelector.values[i] <== rateLimits[i];
    }
    limitSelector.index <== newOriginClass;
    signal limit;
    limit <== limitSelector.out;
    
    component limitCheck = ZKLessThan(32);
    limitCheck.in[0] <== currentCount;
    limitCheck.in[1] <== limit;
    
    limitCheck.out === 1;
    rateLimitOk <== 1;
    
    component incrementer = IncrementAt(7, COUNTER_MAX());
    for (var i = 0; i < 7; i++) {
        incrementer.values[i] <== prevCounters[i];
    }
    incrementer.index <== newOriginClass;
    
    signal newCounters[7];
    for (var i = 0; i < 7; i++) {
        newCounters[i] <== incrementer.newValues[i];
    }
    
    component newHasher = PoseidonHash8();
    newHasher.in[0] <== epochId;
    for (var i = 0; i < 7; i++) {
        newHasher.in[i + 1] <== newCounters[i];
    }
    
    newCounterCommitment <== newHasher.out;
}