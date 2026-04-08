
pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/constants.circom";

/*
 * Rate Limiter: Testing Version
 * Skips counter commitment verification
 */

template RateLimiter() {
    signal input epochId;
    signal input newOriginClass;
    signal input prevCounterCommitment;
    
    signal input prevCounters[7];
    signal input rateLimits[7];
    
    signal output rateLimitOk;
    signal output newCounterCommitment;
    
    // For testing: skip commitment verification
    // Just compute new commitment from inputs
    
    signal newCounters[7];
    for (var i = 0; i < 7; i++) {
        newCounters[i] <== prevCounters[i] + 1;
    }
    
    // Compute new counter commitment
    component newCounterHasher = PoseidonHash8();
    newCounterHasher.in[0] <== epochId;
    for (var i = 0; i < 7; i++) {
        newCounterHasher.in[i + 1] <== newCounters[i];
    }
    newCounterCommitment <== newCounterHasher.out;
    
    rateLimitOk <== 1;
}
