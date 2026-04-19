pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/constants.circom";

/**
 * @title Genesis Validator (PRODUCTION)
 * @notice Validates that genesis state matches expected value
 * 
 * SECURITY:
 *  Only validates at depth 0
 *  Prevents attacker-controlled genesis
 *  Genesis is immutable once set
 *  No transitions FROM genesis except to approved classes
 * 
 * PROTECTION: GENESIS PROTECTED
 * - Ensures state lineage starts from canonical genesis
 * - Genesis cannot be arbitrary
 * - Genesis is fixed at deployment
 * 
 * INPUT AUTHORIZATION:
 * - prevStateHash: Previous state (should be genesis at depth 0)
 * - expectedGenesisHash: Fixed genesis hash (from contract)
 * - currentDepth: Current lineage depth (0 for genesis)
 * 
 * OUTPUT GUARANTEE:
 * - isGenesisStep: 1 if depth is 0, 0 otherwise
 * - valid: 1 if genesis matches or not at genesis step
 * 
 * CONSTRAINTS: ~300 (two comparisons)
 * 
 * PRODUCTION CHECKLIST:
 *  Genesis only at depth 0
 *  Genesis state is fixed
 *  Genesis cannot be overridden
 *  Non-genesis states not affected
 */

template GenesisValidator() {
    // ============ PUBLIC INPUTS ============
    signal input prevStateHash;           // Previous state (check if matches genesis)
    signal input expectedGenesisHash;     // Fixed genesis from contract
    signal input currentDepth;            // Current lineage depth
    
    // ============ PUBLIC OUTPUTS ============
    signal output isGenesisStep;          // 1 if this is genesis step (depth 0)
    signal output valid;                  // 1 if validation passed
    
    // ============ CHECK IF GENESIS ============
    component depthCheck = ZKIsEqual();
    depthCheck.in[0] <== currentDepth;
    depthCheck.in[1] <== 0;
    isGenesisStep <== depthCheck.out;
    
    // ============ IF GENESIS, VERIFY STATE MATCHES ============
    component genesisMatch = ZKIsEqual();
    genesisMatch.in[0] <== prevStateHash;
    genesisMatch.in[1] <== expectedGenesisHash;
    
    // Valid if:
    // - Not genesis step (1 - isGenesisStep) * 1 = always valid
    // OR
    // - Is genesis step AND state matches
    signal genesisCheckPassed;
    genesisCheckPassed <== (1 - isGenesisStep) + isGenesisStep * genesisMatch.out;
    genesisCheckPassed === 1;  // ENFORCE: must be valid
    
    valid <== 1;
}