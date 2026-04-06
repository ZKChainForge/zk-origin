pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsa.circom";
include "../lib/comparators.circom";
include "../lib/merkle.circom";

// Bridge Authentication: Verify cross-chain attestation
template BridgeAuthCircuit(ATTESTATION_DEPTH) {
    signal input sourceChainId;
    signal input expectedSourceChain;
    signal input stateRoot;
    signal input bridgeSignature[2];               // (R, S)
    signal input bridgePublicKey[2];               // (X, Y)
    signal input merkleProof[ATTESTATION_DEPTH];
    signal input merkleIndices[ATTESTATION_DEPTH];
    signal input expectedRoot;
    signal output valid;
    
    // 1. Verify chain ID matches
    component chainMatch = IsEqual();
    chainMatch.in[0] <== sourceChainId;
    chainMatch.in[1] <== expectedSourceChain;
    chainMatch.out === 1;
    
    // 2. Verify bridge signature on state root
    component bridgeVerifier = EdDSAVerifier();
    bridgeVerifier.M <== stateRoot;
    bridgeVerifier.Ax <== bridgePublicKey[0];
    bridgeVerifier.Ay <== bridgePublicKey[1];
    bridgeVerifier.R8x <== bridgeSignature[0];
    bridgeVerifier.R8y <== bridgeSignature[1];
    bridgeVerifier.enabled <== 1;
    bridgeVerifier.valid === 1;
    
    // 3. Verify state root is in bridge's commitment tree
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
]} = BridgeAuthCircuit(6);