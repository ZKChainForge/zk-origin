pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";

template UserAuthCircuit() {
    signal input messageHash;
    signal input signatureCommitment;
    signal input signatureR;
    signal input signatureS;
    signal input publicKeyX;
    signal input publicKeyY;
    signal output valid;
    signal output computedCommitment;

    component commitHasher = PoseidonHash5();
    commitHasher.in[0] <== signatureR;
    commitHasher.in[1] <== signatureS;
    commitHasher.in[2] <== publicKeyX;
    commitHasher.in[3] <== publicKeyY;
    commitHasher.in[4] <== messageHash;

    computedCommitment <== commitHasher.out;

    component eq = IsEqual();
    eq.in[0] <== computedCommitment;
    eq.in[1] <== signatureCommitment;

    valid <== eq.out;
}

component main {public [messageHash, signatureCommitment]} = UserAuthCircuit();
