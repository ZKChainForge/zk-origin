pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/eddsaposeidon.circom";
include "../../node_modules/circomlib/circuits/bitify.circom";

template UserAuth() {
    signal input messageHash;
    signal input publicKeyX;
    signal input publicKeyY;
    signal input signatureR8x;
    signal input signatureR8y;
    signal input signatureS;
    
    signal output valid;
    
    component sigVerifier = EdDSAPoseidonVerifier();
    sigVerifier.enabled <== 1;
    sigVerifier.Ax <== publicKeyX;
    sigVerifier.Ay <== publicKeyY;
    sigVerifier.R8x <== signatureR8x;
    sigVerifier.R8y <== signatureR8y;
    sigVerifier.S <== signatureS;
    sigVerifier.M <== messageHash;
    
    valid <== 1;
}

