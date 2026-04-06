pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../utils/validators.circom";

template LineageStep() {
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal output newLineageCommitment;

    component prevOriginValidator = ValidOriginClass();
    prevOriginValidator.origin <== prevOriginClass;
    prevOriginValidator.valid === 1;

    component newOriginValidator = ValidOriginClass();
    newOriginValidator.origin <== newOriginClass;
    newOriginValidator.valid === 1;

    component transitionHash = PoseidonHash5();
    transitionHash.in[0] <== prevStateHash;
    transitionHash.in[1] <== newStateHash;
    transitionHash.in[2] <== newOriginClass;
    transitionHash.in[3] <== prevOriginClass;
    transitionHash.in[4] <== epochId;

    component lineageUpdate = PoseidonHash3();
    lineageUpdate.in[0] <== 0;
    lineageUpdate.in[1] <== transitionHash.out;
    lineageUpdate.in[2] <== 0;

    newLineageCommitment <== lineageUpdate.out;
}

component main {public [prevStateHash, newStateHash, epochId, prevOriginClass, newOriginClass]} = LineageStep();
