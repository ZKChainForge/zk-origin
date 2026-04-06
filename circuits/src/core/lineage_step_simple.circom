
pragma circom 2.1.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

template LineageStepSimple() {
    signal input prev_state_hash;
    signal input new_state_hash;
    signal input prev_lineage_commitment;
    signal input prev_origin;
    signal input new_origin;
    signal input prev_depth;
    signal input timestamp;
    
    signal output new_lineage_commitment;
    signal output new_depth;
    
    // Simple validation: ensure states are different
    signal state_diff;
    state_diff <== new_state_hash - prev_state_hash;
    
    // Ensure depth increases
    new_depth <== prev_depth + 1;
    
    // Hash to create new lineage commitment
    component hasher = Poseidon(5);
    hasher.inputs[0] <== prev_lineage_commitment;
    hasher.inputs[1] <== prev_state_hash;
    hasher.inputs[2] <== new_state_hash;
    hasher.inputs[3] <== new_origin;
    hasher.inputs[4] <== timestamp;
    
    new_lineage_commitment <== hasher.out;
}

component main {public [
    prev_state_hash,
    new_state_hash,
    prev_lineage_commitment
]} = LineageStepSimple();
