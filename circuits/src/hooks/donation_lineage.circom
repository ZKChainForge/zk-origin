
pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../lib/constants.circom";
include "../lib/validators.circom";
include "../core/policy_verifier.circom";
include "../core/rate_limiter.circom";

template DonationLineageCircuit(POLICY_MERKLE_DEPTH) {

    // ============ PUBLIC INPUTS ============
    signal input poolId;
    signal input donationAmount;
    signal input prevStateHash;
    signal input newStateHash;
    signal input prevLineageCommitment;
    signal input newLineageCommitment;
    signal input prevCounterCommitment;
    signal input newCounterCommitment;
    signal input policyRoot;
    signal input epochId;
    signal input expectedGenesisHash;
    signal input authMessageHash;

    // ============ PRIVATE INPUTS ============
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input prevDepth;
    signal input nonce;
    signal input prevNonce;
    signal input timestamp;
    signal input prevTimestamp;
    signal input prevEpochId;
    signal input policyProof[POLICY_MERKLE_DEPTH];
    signal input policyIndices[POLICY_MERKLE_DEPTH];
    signal input prevCounters[7];
    signal input rateLimits[7];

    // ============ OUTPUT ============
    signal output donationValid;

    // ============ STEP 1: VALIDATE ORIGIN CLASSES ============
    component prevClassValidator = ValidOriginClass();
    prevClassValidator.origin <== prevOriginClass;
    prevClassValidator.valid === 1;

    component newClassValidator = ValidOriginClass();
    newClassValidator.origin <== newOriginClass;
    newClassValidator.valid === 1;

    // ============ STEP 2: ENFORCE DONATION ORIGIN = USER (1) ============
    component originEnforcer = ZKIsEqual();
    originEnforcer.in[0] <== newOriginClass;
    originEnforcer.in[1] <== 1;
    originEnforcer.out === 1;

    // ============ STEP 3: VALIDATE DONATION AMOUNT > 0 ============
    component amountCheck = ZKGreaterThan(64);
    amountCheck.in[0] <== donationAmount;
    amountCheck.in[1] <== 0;
    amountCheck.out === 1;

    // ============ STEP 4: NONCE VALIDATION ============
    component nonceCheck = ZKIsEqual();
    nonceCheck.in[0] <== nonce;
    nonceCheck.in[1] <== prevNonce + 1;
    nonceCheck.out === 1;

    component nonceValidator = ValidNonce();
    nonceValidator.nonce <== nonce;
    nonceValidator.valid === 1;

    // ============ STEP 5: STATE CHANGED ============
    component stateDiff = ZKIsEqual();
    stateDiff.in[0] <== prevStateHash;
    stateDiff.in[1] <== newStateHash;
    signal stateChanged;
    stateChanged <== 1 - stateDiff.out;
    stateChanged === 1;

    // ============ STEP 6: POLICY CHECK ============
    component policyVerifier = PolicyVerifier(POLICY_MERKLE_DEPTH);
    policyVerifier.prevOriginClass <== prevOriginClass;
    policyVerifier.newOriginClass <== newOriginClass;
    policyVerifier.policyRoot <== policyRoot;
    for (var i = 0; i < POLICY_MERKLE_DEPTH; i++) {
        policyVerifier.policyProof[i] <== policyProof[i];
        policyVerifier.policyIndices[i] <== policyIndices[i];
    }
    policyVerifier.isAllowed === 1;

    // ============ STEP 7: RATE LIMITER ============
    component rateLimiter = RateLimiter();
    rateLimiter.epochId <== epochId;
    rateLimiter.newOriginClass <== newOriginClass;
    rateLimiter.prevCounterCommitment <== prevCounterCommitment;
    for (var i = 0; i < 7; i++) {
        rateLimiter.prevCounters[i] <== prevCounters[i];
        rateLimiter.rateLimits[i] <== rateLimits[i];
    }
    rateLimiter.rateLimitOk === 1;

    component counterCheck = ZKIsEqual();
    counterCheck.in[0] <== rateLimiter.newCounterCommitment;
    counterCheck.in[1] <== newCounterCommitment;
    counterCheck.out === 1;

    // ============ STEP 8: COMPUTE TRANSITION HASH ============
    component transitionHasher = PoseidonHash6();
    transitionHasher.in[0] <== prevStateHash;
    transitionHasher.in[1] <== newStateHash;
    transitionHasher.in[2] <== newOriginClass;
    transitionHasher.in[3] <== epochId;
    transitionHasher.in[4] <== timestamp;
    transitionHasher.in[5] <== nonce;
    signal transitionHash;
    transitionHash <== transitionHasher.out;

    // ============ STEP 9: COMPUTE NEW LINEAGE COMMITMENT ============
    component lineageHasher = PoseidonHash3();
    lineageHasher.in[0] <== prevLineageCommitment;
    lineageHasher.in[1] <== transitionHash;
    lineageHasher.in[2] <== prevDepth + 1;
    signal computedLineage;
    computedLineage <== lineageHasher.out;

    component lineageCheck = ZKIsEqual();
    lineageCheck.in[0] <== computedLineage;
    lineageCheck.in[1] <== newLineageCommitment;
    lineageCheck.out === 1;

    // ============ STEP 10: BIND TO POOL ID ============
    component poolBinding = PoseidonHash2();
    poolBinding.in[0] <== computedLineage;
    poolBinding.in[1] <== poolId;
    signal poolBoundCommitment;
    poolBoundCommitment <== poolBinding.out;

    donationValid <== 1;
}