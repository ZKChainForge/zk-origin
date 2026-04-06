pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

template LineageStep() {
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal output newLineageCommitment;

    // Just hash the inputs
    component hasher = Poseidon(5);
    hasher.inputs[0] <== prevStateHash;
    hasher.inputs[1] <== newStateHash;
    hasher.inputs[2] <== prevOriginClass;
    hasher.inputs[3] <== newOriginClass;
    hasher.inputs[4] <== epochId;

    newLineageCommitment <== hasher.out;
}

component main {public [prevStateHash, newStateHash, epochId, prevOriginClass, newOriginClass]} = LineageStep();
