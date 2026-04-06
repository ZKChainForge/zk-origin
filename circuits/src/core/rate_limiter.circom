pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/poseidon.circom";
include "../lib/selector.circom";

// Rate Limiter: Verify and enforce per-origin-class rate limits
template RateLimiter() {
    signal input epochId;
    signal input newOriginClass;
    signal input prevCounters[6];                  // Current counters for each origin class
    signal input prevCounterCommitment;            // Hash of previous counters
    signal input rateLimits[6];                    // Rate limit for each origin class
    signal output rateLimitOk;
    signal output newCounterCommitment;
    
    // 1. Verify counter commitment is correct
    component counterHasher = PoseidonHash7();
    counterHasher.in[0] <== prevCounters[0];
    counterHasher.in[1] <== prevCounters[1];
    counterHasher.in[2] <== prevCounters[2];
    counterHasher.in[3] <== prevCounters[3];
    counterHasher.in[4] <== prevCounters[4];
    counterHasher.in[5] <== prevCounters[5];
    counterHasher.in[6] <== epochId;
    counterHasher.out === prevCounterCommitment;
    
    // 2. Get current counter for new origin class
    component counterSelector = Selector(6);
    counterSelector.values[0] <== prevCounters[0];
    counterSelector.values[1] <== prevCounters[1];
    counterSelector.values[2] <== prevCounters[2];
    counterSelector.values[3] <== prevCounters[3];
    counterSelector.values[4] <== prevCounters[4];
    counterSelector.values[5] <== prevCounters[5];
    counterSelector.index <== newOriginClass;
    signal currentCounter <== counterSelector.out;
    
    // 3. Get rate limit for new origin class
    component limitSelector = Selector(6);
    limitSelector.values[0] <== rateLimits[0];
    limitSelector.values[1] <== rateLimits[1];
    limitSelector.values[2] <== rateLimits[2];
    limitSelector.values[3] <== rateLimits[3];
    limitSelector.values[4] <== rateLimits[4];
    limitSelector.values[5] <== rateLimits[5];
    limitSelector.index <== newOriginClass;
    signal currentLimit <== limitSelector.out;
    
    // 4. Check rate limit not exceeded
    component limitCheck = LessThan(32);
    limitCheck.in[0] <== currentCounter;
    limitCheck.in[1] <== currentLimit;
    limitCheck.out === 1;  // MUST be under limit
    
    // 5. Increment counter for new origin class
    signal newCounters[6];
    for (var i = 0; i < 6; i++) {
        if (i == 0) {
            component eq0 = IsEqual();
            eq0.in[0] <== i;
            eq0.in[1] <== newOriginClass;
            newCounters[i] <== prevCounters[i] + eq0.out;
        } else if (i == 1) {
            component eq1 = IsEqual();
            eq1.in[0] <== i;
            eq1.in[1] <== newOriginClass;
            newCounters[i] <== prevCounters[i] + eq1.out;
        } else if (i == 2) {
            component eq2 = IsEqual();
            eq2.in[0] <== i;
            eq2.in[1] <== newOriginClass;
            newCounters[i] <== prevCounters[i] + eq2.out;
        } else if (i == 3) {
            component eq3 = IsEqual();
            eq3.in[0] <== i;
            eq3.in[1] <== newOriginClass;
            newCounters[i] <== prevCounters[i] + eq3.out;
        } else if (i == 4) {
            component eq4 = IsEqual();
            eq4.in[0] <== i;
            eq4.in[1] <== newOriginClass;
            newCounters[i] <== prevCounters[i] + eq4.out;
        } else {
            component eq5 = IsEqual();
            eq5.in[0] <== i;
            eq5.in[1] <== newOriginClass;
            newCounters[i] <== prevCounters[i] + eq5.out;
        }
    }
    
    // 6. Compute new counter commitment
    component newCounterHasher = PoseidonHash7();
    newCounterHasher.in[0] <== newCounters[0];
    newCounterHasher.in[1] <== newCounters[1];
    newCounterHasher.in[2] <== newCounters[2];
    newCounterHasher.in[3] <== newCounters[3];
    newCounterHasher.in[4] <== newCounters[4];
    newCounterHasher.in[5] <== newCounters[5];
    newCounterHasher.in[6] <== epochId;
    newCounterCommitment <== newCounterHasher.out;
    
    rateLimitOk <== 1;
}

component main {public [
    epochId,
    newOriginClass,
    prevCounterCommitment,
    policyRoot
]} = RateLimiter();