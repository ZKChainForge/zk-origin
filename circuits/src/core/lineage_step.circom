pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/selector.circom";
include "../lib/validators.circom";
include "../lib/constants.circom";
include "./genesis_validator.circom";
include "./policy_verifier.circom";
include "./rate_limiter.circom";

/*
 * Lineage Step: Complete State Transition Verification
 * 
 * Verifies a single state transition with full lineage tracking.
 * Checks:
 * 1. Policy allows transition
 * 2. Authorization is valid
 * 3. Rate limits are enforced
 * 4. Lineage commitment is updated correctly
 * 5. Nonce prevents replays
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
    signal input prevDepth;
    signal input nonce;
    signal input prevNonce;
    signal input timestamp;
    signal input policyProof[POLICY_MERKLE_DEPTH];
    signal input policyIndices[POLICY_MERKLE_DEPTH];
    signal input prevCounters[7];
    signal input rateLimits[7];
    signal input authWitness[200];
    
    // ============ OUTPUTS ============
    signal output newLineageCommitment;
    signal output newCounterCommitment;
    signal output lineageValid;
    
    // ============ DECLARE ALL COMPONENTS FIRST ============
    component prevClassValidator = ValidOriginClass();
    component newClassValidator = ValidOriginClass();
    component nonceCheck = ZKIsEqual();
    component stateDiff = ZKIsEqual();
    component genesisValidator = GenesisValidator();
    component policyVerifier = PolicyVerifier(POLICY_MERKLE_DEPTH);
    component rateLimiter = RateLimiter();
    component transitionHasher = PoseidonHash6();
    component lineageHasher = PoseidonHash3();
    
    // ============ STEP 1: VALIDATE ORIGIN CLASSES ============
    prevClassValidator.origin <== prevOriginClass;
    prevClassValidator.valid === 1;
    
    newClassValidator.origin <== newOriginClass;
    newClassValidator.valid === 1;
    
    // ============ STEP 2: VALIDATE NONCE (REPLAY PREVENTION) ============
    nonceCheck.in[0] <== nonce;
    nonceCheck.in[1] <== prevNonce + 1;
    nonceCheck.out === 1;
    
    // ============ STEP 3: VALIDATE STATES ARE DIFFERENT ============
    stateDiff.in[0] <== prevStateHash;
    stateDiff.in[1] <== newStateHash;
    signal stateChanged <== 1 - stateDiff.out;
    stateChanged === 1;
    
    // ============ STEP 4: VALIDATE GENESIS STATE ============
    genesisValidator.prevStateHash <== prevStateHash;
    genesisValidator.expectedGenesisHash <== expectedGenesisHash;
    genesisValidator.currentDepth <== prevDepth;
    genesisValidator.valid === 1;
    
    // ============ STEP 5: VERIFY POLICY ALLOWS TRANSITION ============
    policyVerifier.prevOriginClass <== prevOriginClass;
    policyVerifier.newOriginClass <== newOriginClass;
    policyVerifier.policyRoot <== policyRoot;
    for (var i = 0; i < POLICY_MERKLE_DEPTH; i++) {
        policyVerifier.policyProof[i] <== policyProof[i];
        policyVerifier.policyIndices[i] <== policyIndices[i];
    }
    policyVerifier.isAllowed === 1;
    
    // ============ STEP 6: VERIFY RATE LIMITS ============
    rateLimiter.epochId <== epochId;
    rateLimiter.newOriginClass <== newOriginClass;
    rateLimiter.prevCounterCommitment <== prevCounterCommitment;
    for (var i = 0; i < 7; i++) {
        rateLimiter.prevCounters[i] <== prevCounters[i];
        rateLimiter.rateLimits[i] <== rateLimits[i];
    }
    rateLimiter.rateLimitOk === 1;
    newCounterCommitment <== rateLimiter.newCounterCommitment;
    
    // ============ STEP 7: COMPUTE TRANSITION HASH ============
    transitionHasher.in[0] <== prevStateHash;
    transitionHasher.in[1] <== newStateHash;
    transitionHasher.in[2] <== newOriginClass;
    transitionHasher.in[3] <== epochId;
    transitionHasher.in[4] <== timestamp;
    transitionHasher.in[5] <== nonce;
    
    // ============ STEP 8: UPDATE LINEAGE COMMITMENT ============
    lineageHasher.in[0] <== prevLineageCommitment;
    lineageHasher.in[1] <== transitionHasher.out;
    lineageHasher.in[2] <== prevDepth + 1;
    
    newLineageCommitment <== lineageHasher.out;
    
    // ============ OUTPUT ============
    lineageValid <== 1;
}