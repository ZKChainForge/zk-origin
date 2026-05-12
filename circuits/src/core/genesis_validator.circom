pragma circom 2.1.0;

include "../lib/comparators.circom";

template GenesisValidator() {
    signal input prevStateHash;
    signal input expectedGenesisHash;
    signal input currentDepth;
    
    signal output isGenesisStep;
    signal output valid;
    
    component depthCheck = ZKIsEqual();
    depthCheck.in[0] <== currentDepth;
    depthCheck.in[1] <== 0;
    isGenesisStep <== depthCheck.out;
    
    component genesisMatch = ZKIsEqual();
    genesisMatch.in[0] <== prevStateHash;
    genesisMatch.in[1] <== expectedGenesisHash;
    
    signal genesisCheckPassed;
    genesisCheckPassed <== (1 - isGenesisStep) + isGenesisStep * genesisMatch.out;
    genesisCheckPassed === 1;
    
    valid <== 1;
}