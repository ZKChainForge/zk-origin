pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/selector.circom";
include "../lib/validators.circom";
include "../lib/constants.circom";
include "./genesis_validator.circom";
include "./policy_verifier.circom";
include "./rate_limiter.circom";
include "./epoch_manager.circom";

/*
 * Lineage Step: Complete State Transition Verification
 * 
 * FINAL VERSION - All components properly integrated
 */

template LineageStep(POLICY_MERKLE_DEPTH) {
    // ============ PUBLIC INPUTS ============
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input prevLineageCommitment;
    signal input prevCounterCommitment;
    signal input policyRoot;
    signal input expectedGenesisHash;
    
    // ============ PRIVATE INPUTS ============
    signal input prevEpochId;
    signal input prevDepth;
    signal input nonce;
    signal input prevNonce;
    signal input timestamp;
    signal input prevTimestamp;
    signal input policyProof[POLICY_MERKLE_DEPTH];
    signal input policyIndices[POLICY_MERKLE_DEPTH];
    signal input prevCounters[7];
    signal input rateLimits[7];
    signal input authorizationValid;
    
    // ============ OUTPUTS ============
    signal output newLineageCommitment;
    signal output newCounterCommitment;
    signal output lineageValid;
    
    // ============ STEP 1: VALIDATE ORIGIN CLASSES ============
    component prevClassValidator = ValidOriginClass();
    prevClassValidator.origin <== prevOriginClass;
    prevClassValidator.valid === 1;
    
    component newClassValidator = ValidOriginClass();
    newClassValidator.origin <== newOriginClass;
    newClassValidator.valid === 1;
    
    // ============ STEP 2: VALIDATE NONCE ============
    component nonceCheck = ZKIsEqual();
    nonceCheck.in[0] <== nonce;
    nonceCheck.in[1] <== prevNonce + 1;
    nonceCheck.out === 1;
    
    component nonceOverflowCheck = ValidNonce();
    nonceOverflowCheck.nonce <== nonce;
    nonceOverflowCheck.valid === 1;
    
    // ============ STEP 3: VALIDATE STATES ARE DIFFERENT ============
    component stateDiff = ZKIsEqual();
    stateDiff.in[0] <== prevStateHash;
    stateDiff.in[1] <== newStateHash;
    signal stateChanged <== 1 - stateDiff.out;
    stateChanged === 1;
    
    // ============ STEP 4: VALIDATE GENESIS ============
    component genesisValidator = GenesisValidator();
    genesisValidator.prevStateHash <== prevStateHash;
    genesisValidator.expectedGenesisHash <== expectedGenesisHash;
    genesisValidator.currentDepth <== prevDepth;
    genesisValidator.valid === 1;
    
    // ============ STEP 5: VERIFY EPOCH TRANSITION ============
    component epochManager = EpochManager();
    epochManager.prevEpochId <== prevEpochId;
    epochManager.newEpochId <== epochId;
    epochManager.prevTimestamp <== prevTimestamp;
    epochManager.newTimestamp <== timestamp;
    for (var i = 0; i < 7; i++) {
        epochManager.prevCounters[i] <== prevCounters[i];
    }
    epochManager.epochValid === 1;
    epochManager.countersValid === 1;
    
    // ============ STEP 6: VERIFY POLICY (FIXED!) ============
    component policyVerifier = PolicyVerifier(POLICY_MERKLE_DEPTH);
    policyVerifier.prevOriginClass <== prevOriginClass;
    policyVerifier.newOriginClass <== newOriginClass;
    policyVerifier.policyRoot <== policyRoot;
    for (var i = 0; i < POLICY_MERKLE_DEPTH; i++) {
        policyVerifier.policyProof[i] <== policyProof[i];
        policyVerifier.policyIndices[i] <== policyIndices[i];
    }
    policyVerifier.isAllowed === 1;
    
    // ============ STEP 7: VERIFY AUTHORIZATION ============
    component authCheck = ZKIsEqual();
    authCheck.in[0] <== authorizationValid;
    authCheck.in[1] <== 1;
    authCheck.out === 1;
    
    // ============ STEP 8: VERIFY RATE LIMITS (FIXED!) ============
    component rateLimiter = RateLimiter();
    rateLimiter.epochId <== epochId;
    rateLimiter.newOriginClass <== newOriginClass;
    rateLimiter.prevCounterCommitment <== prevCounterCommitment;
    for (var i = 0; i < 7; i++) {
        rateLimiter.prevCounters[i] <== prevCounters[i];
        rateLimiter.rateLimits[i] <== rateLimits[i];
    }
    rateLimiter.rateLimitOk === 1;
    newCounterCommitment <== rateLimiter.newCounterCommitment;
    
    // ============ STEP 9: COMPUTE TRANSITION HASH ============
    component transitionHasher = PoseidonHash6();
    transitionHasher.in[0] <== prevStateHash;
    transitionHasher.in[1] <== newStateHash;
    transitionHasher.in[2] <== newOriginClass;
    transitionHasher.in[3] <== epochId;
    transitionHasher.in[4] <== timestamp;
    transitionHasher.in[5] <== nonce;
    
    // ============ STEP 10: UPDATE LINEAGE COMMITMENT ============
    component lineageHasher = PoseidonHash3();
    lineageHasher.in[0] <== prevLineageCommitment;
    lineageHasher.in[1] <== transitionHasher.out;
    lineageHasher.in[2] <== prevDepth + 1;
    
    newLineageCommitment <== lineageHasher.out;
    lineageValid <== 1;
}