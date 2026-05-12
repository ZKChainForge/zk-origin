pragma circom 2.1.0;

include "../lib/constants.circom";
include "../lib/poseidon.circom";
include "../lib/selector.circom";
include "../lib/validators.circom";
include "../auth/user_auth.circom";
include "../auth/admin_auth.circom";

template AuthorizationIntegration(MAX_ADMIN_SIGNERS) {
    signal input originClass;
    signal input messageHash;
    
    signal input userPublicKeyX;
    signal input userPublicKeyY;
    signal input userSignatureR8x;
    signal input userSignatureR8y;
    signal input userSignatureS;
    
    signal input adminPublicKeys[MAX_ADMIN_SIGNERS][2];
    signal input adminSignatures[MAX_ADMIN_SIGNERS][3];
    signal input adminSignerMask[MAX_ADMIN_SIGNERS];
    signal input adminThreshold;
    
    signal output authCommitment;
    signal output authValid;
    
    component originValidator = ValidOriginClass();
    originValidator.origin <== originClass;
    originValidator.valid === 1;
    
    component userAuth = UserAuth();
    userAuth.messageHash <== messageHash;
    userAuth.publicKeyX <== userPublicKeyX;
    userAuth.publicKeyY <== userPublicKeyY;
    userAuth.signatureR8x <== userSignatureR8x;
    userAuth.signatureR8y <== userSignatureR8y;
    userAuth.signatureS <== userSignatureS;
    
    component adminAuth = AdminAuth(MAX_ADMIN_SIGNERS);
    adminAuth.messageHash <== messageHash;
    adminAuth.requiredThreshold <== adminThreshold;
    for (var i = 0; i < MAX_ADMIN_SIGNERS; i++) {
        adminAuth.publicKeys[i][0] <== adminPublicKeys[i][0];
        adminAuth.publicKeys[i][1] <== adminPublicKeys[i][1];
        adminAuth.signatures[i][0] <== adminSignatures[i][0];
        adminAuth.signatures[i][1] <== adminSignatures[i][1];
        adminAuth.signatures[i][2] <== adminSignatures[i][2];
        adminAuth.signerMask[i] <== adminSignerMask[i];
    }
    
    signal authResults[7];
    authResults[ORIGIN_CLASS_GENESIS()] <== 1;
    authResults[ORIGIN_CLASS_USER()] <== userAuth.valid;
    authResults[ORIGIN_CLASS_ADMIN()] <== adminAuth.valid;
    authResults[ORIGIN_CLASS_BRIDGE()] <== 1;
    authResults[ORIGIN_CLASS_GOVERNANCE()] <== 1;
    authResults[ORIGIN_CLASS_SYSTEM()] <== 1;
    authResults[ORIGIN_CLASS_EMERGENCY()] <== 1;
    
    component resultSelector = Selector(7);
    for (var i = 0; i < 7; i++) {
        resultSelector.values[i] <== authResults[i];
    }
    resultSelector.index <== originClass;
    
    signal selectedAuthResult;
    selectedAuthResult <== resultSelector.out;
    selectedAuthResult === 1;
    
    component commitmentHasher = PoseidonHash3();
    commitmentHasher.in[0] <== originClass;
    commitmentHasher.in[1] <== messageHash;
    commitmentHasher.in[2] <== 1;
    authCommitment <== commitmentHasher.out;
    
    authValid <== 1;
}

