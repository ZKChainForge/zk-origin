pragma circom 2.1.0;

include "../lib/constants.circom";
include "../lib/poseidon.circom";
include "../core/lineage_step.circom";
include "../core/auth_integration.circom";

template Main(POLICY_MERKLE_DEPTH, MAX_ADMIN_SIGNERS) {
    
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input prevLineageCommitment;
    signal input prevCounterCommitment;
    signal input policyRoot;
    signal input expectedGenesisHash;
    signal input authMessageHash;
    
    signal input counterValue0;
    signal input counterValue1;
    signal input counterValue2;
    signal input counterValue3;
    signal input counterValue4;
    signal input counterValue5;
    signal input counterValue6;
    
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
    
    signal input userPublicKeyX;
    signal input userPublicKeyY;
    signal input userSignatureR8x;
    signal input userSignatureR8y;
    signal input userSignatureS;
    
    signal input adminPublicKeys[MAX_ADMIN_SIGNERS][2];
    signal input adminSignatures[MAX_ADMIN_SIGNERS][3];
    signal input adminSignerMask[MAX_ADMIN_SIGNERS];
    signal input adminThreshold;
    
    signal output newLineageCommitment;
    signal output newCounterCommitment;
    signal output lineageValid;
    
    component authIntegration = AuthorizationIntegration(MAX_ADMIN_SIGNERS);
    authIntegration.originClass <== newOriginClass;
    authIntegration.messageHash <== authMessageHash;
    authIntegration.userPublicKeyX <== userPublicKeyX;
    authIntegration.userPublicKeyY <== userPublicKeyY;
    authIntegration.userSignatureR8x <== userSignatureR8x;
    authIntegration.userSignatureR8y <== userSignatureR8y;
    authIntegration.userSignatureS <== userSignatureS;
    authIntegration.adminPublicKeys <== adminPublicKeys;
    authIntegration.adminSignatures <== adminSignatures;
    authIntegration.adminSignerMask <== adminSignerMask;
    authIntegration.adminThreshold <== adminThreshold;
    
    component lineageStep = LineageStep(POLICY_MERKLE_DEPTH);
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
    
    for (var i = 0; i < POLICY_MERKLE_DEPTH; i++) {
        lineageStep.policyProof[i] <== policyProof[i];
        lineageStep.policyIndices[i] <== policyIndices[i];
    }
    
    for (var i = 0; i < 7; i++) {
        lineageStep.prevCounters[i] <== prevCounters[i];
        lineageStep.rateLimits[i] <== rateLimits[i];
    }
    
    lineageStep.authorizationCommitment <== authIntegration.authCommitment;
    
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
    expectedGenesisHash,
    authMessageHash,
    counterValue0,
    counterValue1,
    counterValue2,
    counterValue3,
    counterValue4,
    counterValue5,
    counterValue6
]} = Main(6, 15);