pragma circom 2.1.0;

/*
 * ZK-ORIGIN: Simple Lineage Step Circuit (No Merkle Policy)
 * 
 * This is a simplified version with hardcoded policy for testing.
 * Use this for initial development and testing.
 */

include "../lib/poseidon.circom";
include "../lib/comparators.circom";

template LineageStepSimple() {
    // PUBLIC INPUTS

    signal input prev_state_hash;
    signal input new_state_hash;
    signal input prev_lineage_commitment;
    
    
    // PRIVATE INPUTS

    signal input prev_origin;      // 0=Genesis, 1=User, 2=Admin, 3=Bridge
    signal input new_origin;
    signal input prev_depth;
    signal input timestamp;
    
    // PUBLIC OUTPUTS
    
    signal output new_lineage_commitment;
    signal output new_depth;
    
    
    // STEP 1: Validate origin ranges
    
    
    component prevOriginCheck = LessThan(8);
    prevOriginCheck.in[0] <== prev_origin;
    prevOriginCheck.in[1] <== 4;
    prevOriginCheck.out === 1;
    
    component newOriginCheck = LessThan(8);
    newOriginCheck.in[0] <== new_origin;
    newOriginCheck.in[1] <== 4;
    newOriginCheck.out === 1;
    
   
    // STEP 2: Hardcoded Policy Check
   
    
    /*
     * Policy Matrix:
     * Genesis(0) -> User(1): YES
     * Genesis(0) -> Admin(2): YES
     * User(1) -> User(1): YES
     * User(1) -> Admin(2): NO
     * User(1) -> Bridge(3): NO
     * Admin(2) -> User(1): YES
     * Admin(2) -> Admin(2): YES
     * Admin(2) -> Bridge(3): YES
     * Bridge(3) -> User(1): YES
     * Bridge(3) -> Admin(2): NO
     * Bridge(3) -> Bridge(3): NO
     * Any -> Genesis(0): NO (after depth 0)
     */
    
    // Check for disallowed transitions
    component isUser = IsEqual();
    isUser.in[0] <== prev_origin;
    isUser.in[1] <== 1;
    
    component isBridge = IsEqual();
    isBridge.in[0] <== prev_origin;
    isBridge.in[1] <== 3;
    
    component toAdmin = IsEqual();
    toAdmin.in[0] <== new_origin;
    toAdmin.in[1] <== 2;
    
    component toBridge = IsEqual();
    toBridge.in[0] <== new_origin;
    toBridge.in[1] <== 3;
    
    component toGenesis = IsEqual();
    toGenesis.in[0] <== new_origin;
    toGenesis.in[1] <== 0;
    
    component depthPositive = GreaterThan(32);
    depthPositive.in[0] <== prev_depth;
    depthPositive.in[1] <== 0;
    
    // Violations:
    signal userToAdmin;
    userToAdmin <== isUser.out * toAdmin.out;
    
    signal userToBridge;
    userToBridge <== isUser.out * toBridge.out;
    
    signal bridgeToAdmin;
    bridgeToAdmin <== isBridge.out * toAdmin.out;
    
    signal bridgeToBridge;
    bridgeToBridge <== isBridge.out * toBridge.out;
    
    signal invalidGenesisReturn;
    invalidGenesisReturn <== depthPositive.out * toGenesis.out;
    
    // Total violations must be 0
    signal totalViolations;
    totalViolations <== userToAdmin + userToBridge + bridgeToAdmin + bridgeToBridge + invalidGenesisReturn;
    totalViolations === 0;
    
    // ============================================
    // STEP 3: Compute transition hash
    // ============================================
    
    component transitionHash = PoseidonHash4();
    transitionHash.in[0] <== prev_state_hash;
    transitionHash.in[1] <== new_state_hash;
    transitionHash.in[2] <== new_origin;
    transitionHash.in[3] <== timestamp;
    
    // ============================================
    // STEP 4: Update lineage commitment
    // ============================================
    
    component lineageUpdate = PoseidonHash3();
    lineageUpdate.in[0] <== prev_lineage_commitment;
    lineageUpdate.in[1] <== transitionHash.out;
    lineageUpdate.in[2] <== prev_depth + 1;
    
    new_lineage_commitment <== lineageUpdate.out;
    new_depth <== prev_depth + 1;
}

component main {public [
    prev_state_hash,
    new_state_hash,
    prev_lineage_commitment
]} = LineageStepSimple();