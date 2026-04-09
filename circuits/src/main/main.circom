pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/validators.circom";
include "../lib/constants.circom";
include "../core/lineage_step.circom";

/*
 * ZK-ORIGIN Main Circuit
 */

template Main() {
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input prevLineageCommitment;
    signal input prevCounterCommitment;
    signal input policyRoot;
    signal input expectedGenesisHash;
    
    signal input prevEpochId;
    signal input prevDepth;
    signal input nonce;
    signal input prevNonce;
    signal input timestamp;
    signal input prevTimestamp;
    signal input policyProof[6];
    signal input policyIndices[6];
    signal input prevCounters[7];
    signal input rateLimits[7];
    signal input authorizationValid;
    
    signal output newLineageCommitment;
    signal output newCounterCommitment;
    signal output lineageValid;
    
    component lineageStep = LineageStep(6);
    
    lineageStep.prevStateHash <== prevStateHash;
    lineageStep.newStateHash <== newStateHash;
    lineageStep.epochId <== epochId;
    lineageStep.prevOriginClass <== prevOriginClass;
    lineageStep.newOriginClass <== newOriginClass;
    lineageStep.prevLineageCommitment <== prevLineageCommitment;
    lineageStep.prevCounterCommitment <== prevCounterCommitment;
    lineageStep.policyRoot <== policyRoot;
    lineageStep.expectedGenesisHash <== expectedGenesisHash;
    
    lineageStep.prevEpochId <== prevEpochId;
    lineageStep.prevDepth <== prevDepth;
    lineageStep.nonce <== nonce;
    lineageStep.prevNonce <== prevNonce;
    lineageStep.timestamp <== timestamp;
    lineageStep.prevTimestamp <== prevTimestamp;
    
    for (var i = 0; i < 6; i++) {
        lineageStep.policyProof[i] <== policyProof[i];
        lineageStep.policyIndices[i] <== policyIndices[i];
    }
    for (var i = 0; i < 7; i++) {
        lineageStep.prevCounters[i] <== prevCounters[i];
        lineageStep.rateLimits[i] <== rateLimits[i];
    }
    lineageStep.authorizationValid <== authorizationValid;
    
    newLineageCommitment <== lineageStep.newLineageCommitment;
    newCounterCommitment <== lineageStep.newCounterCommitment;
    lineageValid <== lineageStep.lineageValid;
}

// ✅ ONLY THIS FILE HAS component main
component main {public [
    prevStateHash,
    newStateHash,
    epochId,
    prevOriginClass,
    newOriginClass,
    prevLineageCommitment,
    prevCounterCommitment,
    policyRoot,
    expectedGenesisHash
]} = Main();