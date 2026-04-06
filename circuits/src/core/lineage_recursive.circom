pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "./lineage_step.circom";

template RecursiveLineageStep() {
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal output newLineageCommitment;

    component step = LineageStep();
    step.prevStateHash <== prevStateHash;
    step.newStateHash <== newStateHash;
    step.epochId <== epochId;
    step.prevOriginClass <== prevOriginClass;
    step.newOriginClass <== newOriginClass;

    newLineageCommitment <== step.newLineageCommitment;
}

component main {public [prevStateHash, newStateHash, epochId, prevOriginClass, newOriginClass]} = RecursiveLineageStep();
