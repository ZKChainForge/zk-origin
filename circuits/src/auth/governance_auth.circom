pragma circom 2.1.0;

include "../lib/comparators.circom";

template GovernanceAuthCircuit() {
    signal input yesVotes;
    signal input noVotes;
    signal output valid;

    signal netVotes;
    netVotes <== yesVotes - noVotes;

    component voteCheck = GreaterThan(32);
    voteCheck.in[0] <== netVotes;
    voteCheck.in[1] <== 0;

    valid <== voteCheck.out;
}

component main {public [yesVotes, noVotes]} = GovernanceAuthCircuit();
