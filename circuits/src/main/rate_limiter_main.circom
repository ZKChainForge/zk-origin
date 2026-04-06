pragma circom 2.1.0;
include "../core/rate_limiter.circom";
component main {public [epochId, newOriginClass]} = RateLimiter();
