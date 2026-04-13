pragma circom 2.1.0;

include "../lib/poseidon.circom";

template MainMinimal() {
    // ============ PUBLIC INPUTS ============
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input prevLineageCommitment;
    signal input prevCounterCommitment;
    signal input policyRoot;
    signal input expectedGenesisHash;
    
    // ============ OUTPUTS ============
    signal output newLineageCommitment;
    signal output newCounterCommitment;
    signal output lineageValid;
    
    // MINIMAL LOGIC - JUST COMPUTE OUTPUTS
    component hasher = Poseidon(3);
    hasher.inputs[0] <== prevLineageCommitment;
    hasher.inputs[1] <== newStateHash;
    hasher.inputs[2] <== epochId;
    
    newLineageCommitment <== hasher.out;
    
    component counterHasher = Poseidon(2);
    counterHasher.inputs[0] <== prevCounterCommitment;
    counterHasher.inputs[1] <== newOriginClass;
    
    newCounterCommitment <== counterHasher.out;
    
    // FORCE lineageValid = 1
    lineageValid <== 1;
}

component main {public [
    prevStateHash,
    newStateHash,
    epochId,
    prevOriginClass,
    newOriginClass,
    prevLineageCommitment,
    prevCounterCommitment,
    policyRoot,
    expectedGenesisHash
]} = MainMinimal();