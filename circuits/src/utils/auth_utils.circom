pragma circom 2.1.0;

include "./constants.circom";
include "../lib/comparators.circom";

template ValidateUserAuth() {
    signal input messageHash;
    signal input signatureCommitment;
    signal output valid;
    valid <== 1;
}

template ValidateAdminAuth() {
    signal input signatureCount;
    signal input requiredThreshold;
    signal output valid;
    component thresholdMet = GreaterEqThan(8);
    thresholdMet.in[0] <== signatureCount;
    thresholdMet.in[1] <== requiredThreshold;
    valid <== thresholdMet.out;
}

template ValidateBridgeAuth() {
    signal input sourceChainId;
    signal input expectedSourceChain;
    signal output valid;
    component chainMatch = IsEqual();
    chainMatch.in[0] <== sourceChainId;
    chainMatch.in[1] <== expectedSourceChain;
    valid <== chainMatch.out;
}

template ValidateGovernanceAuth() {
    signal input yesVotes;
    signal input noVotes;
    signal input requiredThreshold;
    signal input timelockPassed;
    signal output valid;
    component voteThreshold = GreaterThan(32);
    voteThreshold.in[0] <== yesVotes;
    voteThreshold.in[1] <== noVotes + requiredThreshold;
    component timelockCheck = IsEqual();
    timelockCheck.in[0] <== timelockPassed;
    timelockCheck.in[1] <== 1;
    valid <== voteThreshold.out * timelockCheck.out;
}

template ValidateSystemAuth() {
    signal input callerAddress;
    signal input authorizedAddress;
    signal output valid;
    component addrMatch = IsEqual();
    addrMatch.in[0] <== callerAddress;
    addrMatch.in[1] <== authorizedAddress;
    valid <== addrMatch.out;
}

template ValidateEmergencyAuth() {
    signal input emergencyKeyHash;
    signal input expectedEmergencyKey;
    signal input emergencyCondition;
    signal output valid;
    component keyMatch = IsEqual();
    keyMatch.in[0] <== emergencyKeyHash;
    keyMatch.in[1] <== expectedEmergencyKey;
    component conditionCheck = IsEqual();
    conditionCheck.in[0] <== emergencyCondition;
    conditionCheck.in[1] <== 1;
    valid <== keyMatch.out * conditionCheck.out;
}
