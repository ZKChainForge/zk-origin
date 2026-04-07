pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/validators.circom";
include "../lib/constants.circom";
include "../core/lineage_step.circom";

/*
 * ZK-ORIGIN Main Circuit
 * 
 * Complete state transition proof with all validations.
 * Entry point for the proving system.
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
    signal input prevDepth;
    signal input nonce;
    signal input prevNonce;
    signal input timestamp;
    signal input policyProof[6];
    signal input policyIndices[6];
    signal input prevCounters[7];
    signal input rateLimits[7];
    signal input authWitness[200];
    
    // ============ OUTPUTS ============
    signal output newLineageCommitment;
    signal output newCounterCommitment;
    signal output lineageValid;
    
    // ============ DECLARE COMPONENT FIRST ============
    component lineageStep = LineageStep(6);
    
    // ============ PUBLIC INPUTS ============
    lineageStep.prevStateHash <== prevStateHash;
    lineageStep.newStateHash <== newStateHash;
    lineageStep.epochId <== epochId;
    lineageStep.prevOriginClass <== prevOriginClass;
    lineageStep.newOriginClass <== newOriginClass;
    lineageStep.prevLineageCommitment <== prevLineageCommitment;
    lineageStep.prevCounterCommitment <== prevCounterCommitment;
    lineageStep.policyRoot <== policyRoot;
    lineageStep.expectedGenesisHash <== expectedGenesisHash;
    
    // ============ PRIVATE INPUTS ============
    lineageStep.prevDepth <== prevDepth;
    lineageStep.nonce <== nonce;
    lineageStep.prevNonce <== prevNonce;
    lineageStep.timestamp <== timestamp;
    
    for (var i = 0; i < 6; i++) {
        lineageStep.policyProof[i] <== policyProof[i];
        lineageStep.policyIndices[i] <== policyIndices[i];
    }
    for (var i = 0; i < 7; i++) {
        lineageStep.prevCounters[i] <== prevCounters[i];
        lineageStep.rateLimits[i] <== rateLimits[i];
    }
    for (var i = 0; i < 200; i++) {
        lineageStep.authWitness[i] <== authWitness[i];
    }
    
    // ============ OUTPUTS ============
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