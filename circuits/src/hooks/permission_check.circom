
pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/merkle.circom";
include "../lib/constants.circom";
include "../lib/validators.circom";
include "../core/policy_verifier.circom";

template PermissionCheckCircuit(POLICY_MERKLE_DEPTH) {

    // ============ PUBLIC INPUTS ============
    signal input callerStateHash;
    signal input poolId;
    signal input actionType;
    signal input requiredOriginClass;
    signal input lineageCommitment;
    signal input policyRoot;
    signal input epochId;

    // ============ PRIVATE INPUTS ============
    signal input callerOriginClass;
    signal input callerDepth;
    signal input prevOriginClass;
    signal input policyProof[POLICY_MERKLE_DEPTH];
    signal input policyIndices[POLICY_MERKLE_DEPTH];

    // ============ OUTPUT ============
    signal output permissionGranted;

    // ============ STEP 1: VALIDATE ORIGIN CLASS ============
    component classValidator = ValidOriginClass();
    classValidator.origin <== callerOriginClass;
    classValidator.valid === 1;

    // ============ STEP 2: CHECK ORIGIN >= REQUIRED ============
    component permissionCheck = ZKGreaterEqThan(8);
    permissionCheck.in[0] <== callerOriginClass;
    permissionCheck.in[1] <== requiredOriginClass;
    permissionCheck.out === 1;

    // ============ STEP 3: ACTION TYPE RANGE CHECK ============
    component actionRange = ZKLessThan(8);
    actionRange.in[0] <== actionType;
    actionRange.in[1] <== 4;
    actionRange.out === 1;

    // ============ STEP 4: DEPTH > 0 ============
    component depthCheck = ZKGreaterThan(32);
    depthCheck.in[0] <== callerDepth;
    depthCheck.in[1] <== 0;
    depthCheck.out === 1;

    // ============ STEP 5: POLICY TRANSITION VALID ============
    component policyVerifier = PolicyVerifier(POLICY_MERKLE_DEPTH);
    policyVerifier.prevOriginClass <== prevOriginClass;
    policyVerifier.newOriginClass <== callerOriginClass;
    policyVerifier.policyRoot <== policyRoot;
    for (var i = 0; i < POLICY_MERKLE_DEPTH; i++) {
        policyVerifier.policyProof[i] <== policyProof[i];
        policyVerifier.policyIndices[i] <== policyIndices[i];
    }
    policyVerifier.isAllowed === 1;

    // ============ STEP 6: POOL BINDING ============
    component poolHash = PoseidonHash3();
    poolHash.in[0] <== callerStateHash;
    poolHash.in[1] <== poolId;
    poolHash.in[2] <== actionType;
    signal permissionKey;
    permissionKey <== poolHash.out;

    permissionGranted <== 1;
}