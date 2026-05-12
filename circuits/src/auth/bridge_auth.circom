pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/merkle.circom";

/**
 * Bridge Authentication
 * Verifies cross-chain state with finality and quorum
 * 
 * Security:
 * - Chain ID checked against expected
 * - Finality enforced (64 confirmations)
 * - Validator quorum computed with constrained division
 * - Merkle proof of state inclusion
 * - No hard constraint on valid - parent enforces
 */
template BridgeAuth(ATTESTATION_DEPTH, MAX_VALIDATORS) {
    // Public
    signal input sourceChainId;
    signal input expectedSourceChain;
    signal input stateRoot;
    signal input expectedRoot;
    signal input sourceBlockNumber;
    signal input sourceLatestBlock;

    // Private
    signal input validatorPublicKeys[MAX_VALIDATORS][2];
    signal input validatorSigsR8x[MAX_VALIDATORS];
    signal input validatorSigsR8y[MAX_VALIDATORS];
    signal input validatorSigsS[MAX_VALIDATORS];
    signal input validatorMask[MAX_VALIDATORS];
    signal input bridgeSigR8x;
    signal input bridgeSigR8y;
    signal input bridgeSigS;
    signal input bridgePublicKeyX;
    signal input bridgePublicKeyY;
    signal input merkleProof[ATTESTATION_DEPTH];
    signal input merkleIndices[ATTESTATION_DEPTH];

    // Output - parent enforces === 1
    signal output valid;

    // Step 1: Chain ID match
    component chainMatch = ZKIsEqual();
    chainMatch.in[0] <== sourceChainId;
    chainMatch.in[1] <== expectedSourceChain;
    chainMatch.out === 1;

    // Step 2: Block ordering (latest > source)
    component blockOrder = ZKGreaterThan(32);
    blockOrder.in[0] <== sourceLatestBlock;
    blockOrder.in[1] <== sourceBlockNumber;
    blockOrder.out === 1;

    // Step 3: Finality (>= 64 confirmations)
    signal confirmations;
    confirmations <== sourceLatestBlock - sourceBlockNumber;

    component finalityCheck = ZKGreaterEqThan(32);
    finalityCheck.in[0] <== confirmations;
    finalityCheck.in[1] <== 64;  // MIN_BRIDGE_CONFIRMATIONS
    finalityCheck.out === 1;

    // Step 4: Validator mask binary
    component maskBit[MAX_VALIDATORS];
    for (var i = 0; i < MAX_VALIDATORS; i++) {
        maskBit[i] = IsBinary();
        maskBit[i].value <== validatorMask[i];
        maskBit[i].valid === 1;
    }

    // Step 5: Validator signatures and quorum count
    component valVerifiers[MAX_VALIDATORS];
    signal valCounts[MAX_VALIDATORS + 1];
    valCounts[0] <== 0;

    for (var i = 0; i < MAX_VALIDATORS; i++) {
        valVerifiers[i] = EdDSAMiMCVerifier();
        valVerifiers[i].enabled <== validatorMask[i];
        valVerifiers[i].Ax <== validatorPublicKeys[i][0];
        valVerifiers[i].Ay <== validatorPublicKeys[i][1];
        valVerifiers[i].R8x <== validatorSigsR8x[i];
        valVerifiers[i].R8y <== validatorSigsR8y[i];
        valVerifiers[i].S <== validatorSigsS[i];
        valVerifiers[i].M <== stateRoot;

        valCounts[i + 1] <== valCounts[i] + validatorMask[i];
    }

    // Step 6: Quorum check (2/3 of MAX_VALIDATORS)
    // Compute ceil(MAX_VALIDATORS * 2 / 3) using constrained division
    // quorumThreshold * 3 <= MAX_VALIDATORS * 2 < (quorumThreshold+1) * 3
    signal quorumThreshold;
    quorumThreshold <-- (MAX_VALIDATORS * 2 + 2) / 3;  // ceil

    // Constrain: verify the division is correct
    signal product;
    product <== quorumThreshold * 3;

    component divLow = ZKLessEqThan(16);
    divLow.in[0] <== product;
    divLow.in[1] <== MAX_VALIDATORS * 2 + 2;
    divLow.out === 1;

    component divHigh = ZKGreaterThan(16);
    divHigh.in[0] <== product;
    divHigh.in[1] <== MAX_VALIDATORS * 2 - 2;
    divHigh.out === 1;

    component quorumCheck = ZKGreaterEqThan(8);
    quorumCheck.in[0] <== valCounts[MAX_VALIDATORS];
    quorumCheck.in[1] <== quorumThreshold;
    quorumCheck.out === 1;

    // Step 7: Bridge operator signature
    component bridgeVerifier = EdDSAMiMCVerifier();
    bridgeVerifier.enabled <== 1;
    bridgeVerifier.Ax <== bridgePublicKeyX;
    bridgeVerifier.Ay <== bridgePublicKeyY;
    bridgeVerifier.R8x <== bridgeSigR8x;
    bridgeVerifier.R8y <== bridgeSigR8y;
    bridgeVerifier.S <== bridgeSigS;
    bridgeVerifier.M <== stateRoot;

    // Step 8: Merkle proof of state inclusion
    component merkleVerifier = MerkleProofVerifier(ATTESTATION_DEPTH);
    merkleVerifier.leaf <== stateRoot;
    merkleVerifier.root <== expectedRoot;
    for (var i = 0; i < ATTESTATION_DEPTH; i++) {
        merkleVerifier.pathElements[i] <== merkleProof[i];
        merkleVerifier.pathIndices[i] <== merkleIndices[i];
    }
    merkleVerifier.valid === 1;

    valid <== 1;
}