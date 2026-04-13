pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/selector.circom";
include "../lib/constants.circom";

/*
 * Rate Limiter: Enforce Rate Limits Per Origin Class
 * 
 * PRODUCTION VERSION - Actually enforces rate limits
 */

template RateLimiter() {
    signal input epochId;
    signal input newOriginClass;
    signal input prevCounterCommitment;
    
    signal input prevCounters[7];
    signal input rateLimits[7];
    
    signal output rateLimitOk;
    signal output newCounterCommitment;
    
    // ============ STEP 1: VERIFY PREVIOUS COUNTER COMMITMENT ============
    component prevHasher = PoseidonHash8();
    prevHasher.in[0] <== epochId;
    for (var i = 0; i < 7; i++) {
        prevHasher.in[i + 1] <== prevCounters[i];
    }
    
    // ENFORCE COMMITMENT VERIFICATION
    prevHasher.out === prevCounterCommitment;
    
    // ============ STEP 2: SELECT CURRENT COUNTER FOR ORIGIN CLASS ============
    component counterSelector = Selector(7);
    for (var i = 0; i < 7; i++) {
        counterSelector.values[i] <== prevCounters[i];
    }
    counterSelector.index <== newOriginClass;
    signal currentCount <== counterSelector.out;
    
    // ============ STEP 3: SELECT RATE LIMIT FOR ORIGIN CLASS ============
    component limitSelector = Selector(7);
    for (var i = 0; i < 7; i++) {
        limitSelector.values[i] <== rateLimits[i];
    }
    limitSelector.index <== newOriginClass;
    signal limit <== limitSelector.out;
    
    // ============ STEP 4: ENFORCE RATE LIMIT CHECK ============
    component limitCheck = ZKLessThan(32);
    limitCheck.in[0] <== currentCount;
    limitCheck.in[1] <== limit;
    
    // ENFORCE LIMIT CHECK PASSES
    limitCheck.out === 1;
    rateLimitOk <== 1;
    
    // ============ STEP 5: INCREMENT COUNTER ============
    component incrementer = IncrementAt(7, COUNTER_MAX());
    for (var i = 0; i < 7; i++) {
        incrementer.values[i] <== prevCounters[i];
    }
    incrementer.index <== newOriginClass;
    
    signal newCounters[7];
    for (var i = 0; i < 7; i++) {
        newCounters[i] <== incrementer.newValues[i];
    }
    
    // ============ STEP 6: COMPUTE NEW COUNTER COMMITMENT ============
    component newHasher = PoseidonHash8();
    newHasher.in[0] <== epochId;
    for (var i = 0; i < 7; i++) {
        newHasher.in[i + 1] <== newCounters[i];
    }
    
    newCounterCommitment <== newHasher.out;
}