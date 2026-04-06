pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";

template EmergencyAuthCircuit() {
    signal input emergencyKeyHash;
    signal input expectedEmergencyKey;
    signal output valid;

    component keyMatch = IsEqual();
    keyMatch.in[0] <== emergencyKeyHash;
    keyMatch.in[1] <== expectedEmergencyKey;

    valid <== keyMatch.out;
}

component main {public [emergencyKeyHash, expectedEmergencyKey]} = EmergencyAuthCircuit();
