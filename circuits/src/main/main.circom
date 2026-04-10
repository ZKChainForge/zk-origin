pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/validators.circom";
include "../lib/constants.circom";
include "../core/lineage_step.circom";

/*
 * ZK-ORIGIN Main Circuit
 * 
 * Public Inputs (9):
 *   prevStateHash
 *   newStateHash
 *   epochId
 *   prevOriginClass
 *   newOriginClass
 *   prevLineageCommitment
 *   prevCounterCommitment
 *   policyRoot
 *   expectedGenesisHash
 * 
 * Private Inputs (33):
 *   prevEpochId, prevDepth, nonce, prevNonce
 *   timestamp, prevTimestamp
 *   policyProof[6], policyIndices[6]
 *   prevCounters[7], rateLimits[7]
 *   authorizationValid
 * 
 * Outputs (3):
 *   newLineageCommitment
 *   newCounterCommitment
 *   lineageValid
 */

template Main() {
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
    // Epoch and depth tracking
    signal input prevEpochId;
    signal input prevDepth;
    
    // Nonce sequence
    signal input nonce;
    signal input prevNonce;
    
    // Timestamp tracking
    signal input timestamp;
    signal input prevTimestamp;
    
    // Policy verification
    signal input policyProof[6];
    signal input policyIndices[6];
    
    // Counter tracking
    signal input prevCounters[7];
    signal input rateLimits[7];
    
    // Authorization (SINGLE VALUE - not array!)
    signal input authorizationValid;
    
    // ============ OUTPUTS ============
    signal output newLineageCommitment;
    signal output newCounterCommitment;
    signal output lineageValid;
    
    // ============ INSTANTIATE LINEAGE STEP ============
    component lineageStep = LineageStep(6);
    
    // Connect public inputs
    lineageStep.prevStateHash <== prevStateHash;
    lineageStep.newStateHash <== newStateHash;
    lineageStep.epochId <== epochId;
    lineageStep.prevOriginClass <== prevOriginClass;
    lineageStep.newOriginClass <== newOriginClass;
    lineageStep.prevLineageCommitment <== prevLineageCommitment;
    lineageStep.prevCounterCommitment <== prevCounterCommitment;
    lineageStep.policyRoot <== policyRoot;
    lineageStep.expectedGenesisHash <== expectedGenesisHash;
    
    // Connect private inputs
    lineageStep.prevEpochId <== prevEpochId;
    lineageStep.prevDepth <== prevDepth;
    lineageStep.nonce <== nonce;
    lineageStep.prevNonce <== prevNonce;
    lineageStep.timestamp <== timestamp;
    lineageStep.prevTimestamp <== prevTimestamp;
    
    // Connect policy proof arrays
    for (var i = 0; i < 6; i++) {
        lineageStep.policyProof[i] <== policyProof[i];
        lineageStep.policyIndices[i] <== policyIndices[i];
    }
    
    // Connect counter arrays
    for (var i = 0; i < 7; i++) {
        lineageStep.prevCounters[i] <== prevCounters[i];
        lineageStep.rateLimits[i] <== rateLimits[i];
    }
    
    // Connect authorization (single value!)
    lineageStep.authorizationValid <== authorizationValid;
    
    // ============ OUTPUT RESULTS ============
    newLineageCommitment <== lineageStep.newLineageCommitment;
    newCounterCommitment <== lineageStep.newCounterCommitment;
    lineageValid <== lineageStep.lineageValid;
}

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