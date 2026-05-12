
pragma circom 2.1.0;

include "../lib/constants.circom";
include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/validators.circom";
include "../hooks/permission_check.circom";
include "../auth/user_auth.circom";

template MainPermissionHook() {

    // ============ PUBLIC INPUTS ============
    signal input callerStateHash;
    signal input poolId;
    signal input actionType;
    signal input requiredOriginClass;
    signal input lineageCommitment;
    signal input policyRoot;
    signal input epochId;
    signal input authMessageHash;

    // ============ PRIVATE INPUTS ============
    signal input callerOriginClass;
    signal input callerDepth;
    signal input prevOriginClass;
    signal input policyProof[4];
    signal input policyIndices[4];

    // User auth
    signal input userPublicKeyX;
    signal input userPublicKeyY;
    signal input userSignatureR8x;
    signal input userSignatureR8y;
    signal input userSignatureS;

    // ============ STEP 1: USER AUTHORIZATION ============
    component userAuth = UserAuth();
    userAuth.messageHash <== authMessageHash;
    userAuth.publicKeyX <== userPublicKeyX;
    userAuth.publicKeyY <== userPublicKeyY;
    userAuth.signatureR8x <== userSignatureR8x;
    userAuth.signatureR8y <== userSignatureR8y;
    userAuth.signatureS <== userSignatureS;
    userAuth.valid === 1;

    // ============ STEP 2: PERMISSION VERIFICATION ============
    component permissionCircuit = PermissionCheckCircuit(4);
    permissionCircuit.callerStateHash <== callerStateHash;
    permissionCircuit.poolId <== poolId;
    permissionCircuit.actionType <== actionType;
    permissionCircuit.requiredOriginClass <== requiredOriginClass;
    permissionCircuit.lineageCommitment <== lineageCommitment;
    permissionCircuit.policyRoot <== policyRoot;
    permissionCircuit.epochId <== epochId;
    permissionCircuit.callerOriginClass <== callerOriginClass;
    permissionCircuit.callerDepth <== callerDepth;
    permissionCircuit.prevOriginClass <== prevOriginClass;

    for (var i = 0; i < 4; i++) {
        permissionCircuit.policyProof[i] <== policyProof[i];
        permissionCircuit.policyIndices[i] <== policyIndices[i];
    }

    permissionCircuit.permissionGranted === 1;
}

component main {public [
    callerStateHash,
    poolId,
    actionType,
    requiredOriginClass,
    lineageCommitment,
    policyRoot,
    epochId,
    authMessageHash
]} = MainPermissionHook();