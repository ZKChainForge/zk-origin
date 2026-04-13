pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../lib/validators.circom";
include "../lib/constants.circom";

/*
 * Bridge Authentication: Cross-Chain Attestation with Finality
 * 
 * FIXED: Verifies block order to prevent underflow
 */

template BridgeAuth(ATTESTATION_DEPTH, MAX_VALIDATORS) {
    signal input sourceChainId;
    signal input expectedSourceChain;
    signal input stateRoot;
    signal input expectedRoot;
    
    // ============ FINALITY INPUTS ============
    signal input sourceBlockNumber;
    signal input sourceLatestBlock;
    
    // ============ VALIDATOR QUORUM INPUTS ============
    signal input validatorPublicKeys[MAX_VALIDATORS][2];
    signal input validatorSignatures[MAX_VALIDATORS][2];
    signal input validatorMask[MAX_VALIDATORS];
    
    // ============ PRIVATE INPUTS ============
    signal input bridgeSignatureR;
    signal input bridgeSignatureS;
    signal input bridgePublicKeyX;
    signal input bridgePublicKeyY;
    signal input merkleProof[ATTESTATION_DEPTH];
    signal input merkleIndices[ATTESTATION_DEPTH];
    
    signal output valid;
    
    // ============ VERIFY CHAIN ID MATCHES ============
    component chainMatch = ZKIsEqual();
    chainMatch.in[0] <== sourceChainId;
    chainMatch.in[1] <== expectedSourceChain;
    chainMatch.out === 1;
    
    // ============ VERIFY BLOCK ORDER (PREVENTS UNDERFLOW) ============
    component blockOrder = ZKGreaterThan(32);
    blockOrder.in[0] <== sourceLatestBlock;
    blockOrder.in[1] <== sourceBlockNumber;
    blockOrder.out === 1;
    
    // ============ VERIFY FINALITY ============
    signal confirmations;
    confirmations <== sourceLatestBlock - sourceBlockNumber;
    
    component finalityCheck = ZKGreaterEqThan(32);
    finalityCheck.in[0] <== confirmations;
    finalityCheck.in[1] <== MIN_BRIDGE_CONFIRMATIONS();
    finalityCheck.out === 1;
    
    // ============ VERIFY VALIDATOR MASKS ARE BINARY ============
    component maskValidators[MAX_VALIDATORS];
    for (var i = 0; i < MAX_VALIDATORS; i++) {
        maskValidators[i] = IsBinary();
        maskValidators[i].value <== validatorMask[i];
        maskValidators[i].valid === 1;
    }
    
    // ============ VERIFY VALIDATOR QUORUM ============
    component validatorVerifiers[MAX_VALIDATORS];
    signal validSignatures[MAX_VALIDATORS + 1];
    validSignatures[0] <== 0;
    
    for (var i = 0; i < MAX_VALIDATORS; i++) {
        validatorVerifiers[i] = EdDSAVerifier();
        validatorVerifiers[i].M <== stateRoot;
        validatorVerifiers[i].Ax <== validatorPublicKeys[i][0];
        validatorVerifiers[i].Ay <== validatorPublicKeys[i][1];
        validatorVerifiers[i].R8x <== validatorSignatures[i][0];
        validatorVerifiers[i].R8y <== validatorSignatures[i][1];
        
        validSignatures[i + 1] <== validSignatures[i] + 
            validatorVerifiers[i].valid * validatorMask[i];
    }
    
    // ============ COMPUTE QUORUM THRESHOLD (ROUNDS UP) ============
    signal quorumNumerator;
    quorumNumerator <== MAX_VALIDATORS * BRIDGE_QUORUM_NUMERATOR() + BRIDGE_QUORUM_DENOMINATOR() - 1;
    
    signal quorumThreshold;
    quorumThreshold <== quorumNumerator \ BRIDGE_QUORUM_DENOMINATOR();
    
    component quorumCheck = ZKGreaterEqThan(8);
    quorumCheck.in[0] <== validSignatures[MAX_VALIDATORS];
    quorumCheck.in[1] <== quorumThreshold;
    quorumCheck.out === 1;
    
    // ============ VERIFY BRIDGE SIGNATURE ============
    component bridgeVerifier = EdDSAVerifier();
    bridgeVerifier.M <== stateRoot;
    bridgeVerifier.Ax <== bridgePublicKeyX;
    bridgeVerifier.Ay <== bridgePublicKeyY;
    bridgeVerifier.R8x <== bridgeSignatureR;
    bridgeVerifier.R8y <== bridgeSignatureS;
    bridgeVerifier.valid === 1;
    
    // ============ VERIFY MERKLE PROOF ============
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

component main {public [
    sourceChainId,
    expectedSourceChain,
    stateRoot,
    expectedRoot,
    sourceBlockNumber,
    sourceLatestBlock
]} = BridgeAuth(6, 21);