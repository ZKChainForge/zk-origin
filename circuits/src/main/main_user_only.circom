pragma circom 2.1.0;

include "../lib/constants.circom";
include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/validators.circom";
include "../core/lineage_step.circom";
include "../auth/user_auth.circom";

/*
 * ZK-ORIGIN User-Only Hook Circuit
 * Fixed signal names to match UserAuth template
 */

template MainUserOnly() {
    // ============ PUBLIC INPUTS (12) ============
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
    signal input newLineageCommitment;
    signal input newCounterCommitment;

    // ============ PRIVATE INPUTS ============
    signal input prevEpochId;
    signal input prevDepth;
    signal input nonce;
    signal input prevNonce;
    signal input timestamp;
    signal input prevTimestamp;
    signal input policyProof[4];
    signal input policyIndices[4];
    signal input prevCounters[7];
    signal input rateLimits[7];
    signal input userPublicKeyX;
    signal input userPublicKeyY;
    signal input userSignatureR8x;
    signal input userSignatureR8y;
    signal input userSignatureS;

    // ============ USER AUTHORIZATION ============
    component userAuth = UserAuth();
    userAuth.messageHash <== authMessageHash;
    userAuth.publicKeyX <== userPublicKeyX;
    userAuth.publicKeyY <== userPublicKeyY;
    userAuth.signatureR8x <== userSignatureR8x;
    userAuth.signatureR8y <== userSignatureR8y;
    userAuth.signatureS <== userSignatureS;
    userAuth.valid === 1;

    // Authorization commitment
    component authHash = PoseidonHash2();
    authHash.in[0] <== newOriginClass;
    authHash.in[1] <== 1;
    signal authCommitment;
    authCommitment <== authHash.out;

    // ============ LINEAGE VERIFICATION ============
    component step = LineageStep(4);

    step.prevStateHash <== prevStateHash;
    step.newStateHash <== newStateHash;
    step.epochId <== epochId;
    step.prevOriginClass <== prevOriginClass;
    step.newOriginClass <== newOriginClass;
    step.prevLineageCommitment <== prevLineageCommitment;
    step.prevCounterCommitment <== prevCounterCommitment;
    step.policyRoot <== policyRoot;
    step.expectedGenesisHash <== expectedGenesisHash;
    step.prevEpochId <== prevEpochId;
    step.prevDepth <== prevDepth;
    step.nonce <== nonce;
    step.prevNonce <== prevNonce;
    step.timestamp <== timestamp;
    step.prevTimestamp <== prevTimestamp;

    for (var i = 0; i < 4; i++) {
        step.policyProof[i] <== policyProof[i];
        step.policyIndices[i] <== policyIndices[i];
    }

    for (var i = 0; i < 7; i++) {
        step.prevCounters[i] <== prevCounters[i];
        step.rateLimits[i] <== rateLimits[i];
    }

    step.authorizationCommitment <== authCommitment;

    // ============ ENFORCE OUTPUTS ============
    step.newLineageCommitment === newLineageCommitment;
    step.newCounterCommitment === newCounterCommitment;
    step.lineageValid === 1;
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
    newLineageCommitment,
    newCounterCommitment
]} = MainUserOnly();