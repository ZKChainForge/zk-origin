pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/poseidon.circom";
include "../lib/merkle.circom";
include "./policy_verifier.circom";

// Complete Lineage Step: Verify state transition with all validations
template LineageStep(POLICY_MERKLE_DEPTH) {
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input prevLineageCommitment;
    signal input prevCounterCommitment;
    signal input policyRoot;
    signal input policyProof[POLICY_MERKLE_DEPTH];
    signal input policyIndices[POLICY_MERKLE_DEPTH];
    signal input prevCounters[6];
    signal input rateLimits[6];
    signal output newLineageCommitment;
    signal output newCounterCommitment;
    signal output lineageValid;
    
    // 1. Validate origin classes are in range
    component prevClassCheck = LessThan(8);
    prevClassCheck.in[0] <== prevOriginClass;
    prevClassCheck.in[1] <== 6;
    prevClassCheck.out === 1;
    
    component newClassCheck = LessThan(8);
    newClassCheck.in[0] <== newOriginClass;
    newClassCheck.in[1] <== 6;
    newClassCheck.out === 1;
    
    // 2. Verify states are different (state must change)
    component stateDiff = IsEqual();
    stateDiff.in[0] <== prevStateHash;
    stateDiff.in[1] <== newStateHash;
    signal stateChanged <== 1 - stateDiff.out;
    stateChanged === 1;
    
    // 3. Verify policy allows this transition
    component policyVerifier = PolicyVerifier(POLICY_MERKLE_DEPTH);
    policyVerifier.prevOriginClass <== prevOriginClass;
    policyVerifier.newOriginClass <== newOriginClass;
    policyVerifier.policyRoot <== policyRoot;
    for (var i = 0; i < POLICY_MERKLE_DEPTH; i++) {
        policyVerifier.policyProof[i] <== policyProof[i];
        policyVerifier.policyIndices[i] <== policyIndices[i];
    }
    policyVerifier.isAllowed === 1;
    
    // 4. Verify counter commitment
    component counterHasher = PoseidonHash7();
    counterHasher.in[0] <== prevCounters[0];
    counterHasher.in[1] <== prevCounters[1];
    counterHasher.in[2] <== prevCounters[2];
    counterHasher.in[3] <== prevCounters[3];
    counterHasher.in[4] <== prevCounters[4];
    counterHasher.in[5] <== prevCounters[5];
    counterHasher.in[6] <== epochId;
    counterHasher.out === prevCounterCommitment;
    
    // 5. Check rate limit
    component limitSelector = Selector(6);
    limitSelector.values[0] <== rateLimits[0];
    limitSelector.values[1] <== rateLimits[1];
    limitSelector.values[2] <== rateLimits[2];
    limitSelector.values[3] <== rateLimits[3];
    limitSelector.values[4] <== rateLimits[4];
    limitSelector.values[5] <== rateLimits[5];
    limitSelector.index <== newOriginClass;
    signal currentLimit <== limitSelector.out;
    
    component counterSelector = Selector(6);
    counterSelector.values[0] <== prevCounters[0];
    counterSelector.values[1] <== prevCounters[1];
    counterSelector.values[2] <== prevCounters[2];
    counterSelector.values[3] <== prevCounters[3];
    counterSelector.values[4] <== prevCounters[4];
    counterSelector.values[5] <== prevCounters[5];
    counterSelector.index <== newOriginClass;
    signal currentCounter <== counterSelector.out;
    
    component limitCheck = LessThan(32);
    limitCheck.in[0] <== currentCounter;
    limitCheck.in[1] <== currentLimit;
    limitCheck.out === 1;
    
    // 6. Update counters
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
    
    // 7. Compute new counter commitment
    component newCounterHasher = PoseidonHash7();
    newCounterHasher.in[0] <== newCounters[0];
    newCounterHasher.in[1] <== newCounters[1];
    newCounterHasher.in[2] <== newCounters[2];
    newCounterHasher.in[3] <== newCounters[3];
    newCounterHasher.in[4] <== newCounters[4];
    newCounterHasher.in[5] <== newCounters[5];
    newCounterHasher.in[6] <== epochId;
    newCounterCommitment <== newCounterHasher.out;
    
    // 8. Compute transition hash
    component transitionHasher = PoseidonHash5();
    transitionHasher.in[0] <== prevStateHash;
    transitionHasher.in[1] <== newStateHash;
    transitionHasher.in[2] <== newOriginClass;
    transitionHasher.in[3] <== prevOriginClass;
    transitionHasher.in[4] <== epochId;
    
    // 9. Update lineage commitment
    component lineageHasher = PoseidonHash3();
    lineageHasher.in[0] <== prevLineageCommitment;
    lineageHasher.in[1] <== transitionHasher.out;
    lineageHasher.in[2] <== 1;  // Depth increment
    newLineageCommitment <== lineageHasher.out;
    
    lineageValid <== 1;
}

component main {public [
    prevStateHash,
    newStateHash,
    epochId,
    prevOriginClass,
    newOriginClass,
    prevLineageCommitment,
    prevCounterCommitment,
    policyRoot
]} = LineageStep(6);