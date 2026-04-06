pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/selector.circom";
include "../utils/validators.circom";

template RateLimiter() {
    signal input epochId;
    signal input newOriginClass;
    signal input prevCounter;
    signal input rateLimit;
    signal output rateLimitOk;

    component limitCheck = LessThan(32);
    limitCheck.in[0] <== prevCounter;
    limitCheck.in[1] <== rateLimit;
    
    rateLimitOk <== limitCheck.out;
    limitCheck.out === 1;
}

component main {public [epochId, newOriginClass]} = RateLimiter();
