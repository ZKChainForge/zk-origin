
pragma circom 2.1.0;

include "../lib/constants.circom";
include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/validators.circom";
include "../hooks/donation_lineage.circom";
include "../auth/user_auth.circom";

template MainDonationHook() {

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
    signal input policyProof[4];
    signal input policyIndices[4];
    signal input prevCounters[7];
    signal input rateLimits[7];

    // User auth
    signal input userPublicKeyX;
    signal input userPublicKeyY;
    signal input userSignatureR8x;
    signal input userSignatureR8y;
    signal input userSignatureS;

    // ============ STEP 1: USER AUTHORIZATION ============
    component userAuth = UserAuth();
    userAuth.messageHash <== authMessageHash;
    userAuth.publicKeyX <== userPublicKeyX;
    userAuth.publicKeyY <== userPublicKeyY;
    userAuth.signatureR8x <== userSignatureR8x;
    userAuth.signatureR8y <== userSignatureR8y;
    userAuth.signatureS <== userSignatureS;
    userAuth.valid === 1;

    // ============ STEP 2: DONATION LINEAGE VERIFICATION ============
    component donationCircuit = DonationLineageCircuit(4);
    donationCircuit.poolId <== poolId;
    donationCircuit.donationAmount <== donationAmount;
    donationCircuit.prevStateHash <== prevStateHash;
    donationCircuit.newStateHash <== newStateHash;
    donationCircuit.prevLineageCommitment <== prevLineageCommitment;
    donationCircuit.newLineageCommitment <== newLineageCommitment;
    donationCircuit.prevCounterCommitment <== prevCounterCommitment;
    donationCircuit.newCounterCommitment <== newCounterCommitment;
    donationCircuit.policyRoot <== policyRoot;
    donationCircuit.epochId <== epochId;
    donationCircuit.expectedGenesisHash <== expectedGenesisHash;
    donationCircuit.authMessageHash <== authMessageHash;
    donationCircuit.prevOriginClass <== prevOriginClass;
    donationCircuit.newOriginClass <== newOriginClass;
    donationCircuit.prevDepth <== prevDepth;
    donationCircuit.nonce <== nonce;
    donationCircuit.prevNonce <== prevNonce;
    donationCircuit.timestamp <== timestamp;
    donationCircuit.prevTimestamp <== prevTimestamp;
    donationCircuit.prevEpochId <== prevEpochId;

    for (var i = 0; i < 4; i++) {
        donationCircuit.policyProof[i] <== policyProof[i];
        donationCircuit.policyIndices[i] <== policyIndices[i];
    }

    for (var i = 0; i < 7; i++) {
        donationCircuit.prevCounters[i] <== prevCounters[i];
        donationCircuit.rateLimits[i] <== rateLimits[i];
    }

    donationCircuit.donationValid === 1;
}

component main {public [
    poolId,
    donationAmount,
    prevStateHash,
    newStateHash,
    prevLineageCommitment,
    newLineageCommitment,
    prevCounterCommitment,
    newCounterCommitment,
    policyRoot,
    epochId,
    expectedGenesisHash,
    authMessageHash
]} = MainDonationHook();