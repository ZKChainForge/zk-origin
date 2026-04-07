pragma circom 2.1.0;

/*
 * Bridge Authentication: Cross-Chain Attestation Verification
 * 
 * Verifies that a state import from another chain is properly attested
 * by the bridge validators and is included in the bridge's commitment.
 */

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/merkle.circom";

template BridgeAuth(ATTESTATION_DEPTH) {
    // ============ PUBLIC INPUTS ============
    signal input sourceChainId;
    signal input expectedSourceChain;
    signal input stateRoot;
    signal input expectedRoot;
    
    // ============ PRIVATE INPUTS ============
    signal input bridgeSignatureR;
    signal input bridgeSignatureS;
    signal input bridgePublicKeyX;
    signal input bridgePublicKeyY;
    signal input merkleProof[ATTESTATION_DEPTH];
    signal input merkleIndices[ATTESTATION_DEPTH];
    
    // ============ OUTPUT ============
    signal output valid;
    
    // ============ VERIFY CHAIN ID MATCHES ============
    component chainMatch = IsEqual();
    chainMatch.in[0] <== sourceChainId;
    chainMatch.in[1] <== expectedSourceChain;
    chainMatch.out === 1;
    
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
    expectedRoot
]} = BridgeAuth(6);