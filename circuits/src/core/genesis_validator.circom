pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/constants.circom";

/*
 * Genesis State Validator
 * 
 * Validates that the genesis state matches the expected genesis commitment.
 * Prevents attacker-controlled genesis states.
 */

template GenesisValidator() {
    // ============ PUBLIC INPUTS ============
    signal input prevStateHash;
    signal input expectedGenesisHash;
    signal input currentDepth;
    
    // ============ OUTPUT ============
    signal output isGenesisStep;
    signal output valid;
    
    // ============ CHECK IF THIS IS GENESIS ============
    component isGenesis = ZKIsEqual();
    isGenesis.in[0] <== currentDepth;
    isGenesis.in[1] <== 0;
    isGenesisStep <== isGenesis.out;
    
    // ============ IF GENESIS, VERIFY STATE MATCHES ============
    component genesisMatch = ZKIsEqual();
    genesisMatch.in[0] <== prevStateHash;
    genesisMatch.in[1] <== expectedGenesisHash;
    
    // Must match if genesis
    signal genesisCheckPassed;
    genesisCheckPassed <== (1 - isGenesisStep) + isGenesisStep * genesisMatch.out;
    genesisCheckPassed === 1;
    
    valid <== 1;
}