pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/selector.circom";
include "../lib/constants.circom";

/*
 * Rate Limiter: Per-Origin-Class Rate Enforcement
 * 
 * Verifies and enforces rate limits for each origin class.
 * Tracks counters per origin class per epoch.
 * Prevents counter overflow and rate limit bypass.
 */

template RateLimiter() {
    // ============ PUBLIC INPUTS ============
    signal input epochId;
    signal input newOriginClass;
    signal input prevCounterCommitment;
    
    // ============ PRIVATE INPUTS ============
    signal input prevCounters[7];
    signal input rateLimits[7];
    
    // ============ OUTPUTS ============
    signal output rateLimitOk;
    signal output newCounterCommitment;
    
    // ============ DECLARE ALL SIGNALS FIRST ============
    signal newCounters[7];
    component eqComponents[7];
    
    // ============ VERIFY COUNTER COMMITMENT ============
    component counterHasher = PoseidonHash8();
    counterHasher.in[0] <== epochId;
    for (var i = 0; i < 7; i++) {
        counterHasher.in[i + 1] <== prevCounters[i];
    }
    counterHasher.out === prevCounterCommitment;
    
    // ============ GET CURRENT COUNTER FOR NEW ORIGIN ============
    component counterSelector = Selector(7);
    for (var i = 0; i < 7; i++) {
        counterSelector.values[i] <== prevCounters[i];
    }
    counterSelector.index <== newOriginClass;
    signal currentCounter <== counterSelector.out;
    
    // ============ GET RATE LIMIT FOR NEW ORIGIN ============
    component limitSelector = Selector(7);
    for (var i = 0; i < 7; i++) {
        limitSelector.values[i] <== rateLimits[i];
    }
    limitSelector.index <== newOriginClass;
    signal currentLimit <== limitSelector.out;
    
    // ============ CHECK RATE LIMIT NOT EXCEEDED ============
    component limitCheck = ZKLessThan(32);
    limitCheck.in[0] <== currentCounter;
    limitCheck.in[1] <== currentLimit;
    limitCheck.out === 1;
    
    // ============ COMPUTE NEW COUNTER ============
    signal newCounter;
    newCounter <== currentCounter + 1;
    
    // ============ CHECK FOR OVERFLOW ============
    component noOverflow = ZKLessEqThan(32);
    noOverflow.in[0] <== newCounter;
    noOverflow.in[1] <== 4294967295;
    noOverflow.out === 1;
    
    // ============ UPDATE COUNTERS ============
    for (var i = 0; i < 7; i++) {
        eqComponents[i] = ZKIsEqual();
        eqComponents[i].in[0] <== i;
        eqComponents[i].in[1] <== newOriginClass;
        
        newCounters[i] <== prevCounters[i] + eqComponents[i].out;
    }
    
    // ============ COMPUTE NEW COUNTER COMMITMENT ============
    component newCounterHasher = PoseidonHash8();
    newCounterHasher.in[0] <== epochId;
    for (var i = 0; i < 7; i++) {
        newCounterHasher.in[i + 1] <== newCounters[i];
    }
    newCounterCommitment <== newCounterHasher.out;
    
    rateLimitOk <== 1;
}