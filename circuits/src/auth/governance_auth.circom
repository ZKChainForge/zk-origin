pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/poseidon.circom";

// Governance Authentication: Verify proposal voting results
template GovernanceAuthCircuit() {
    signal input proposalId;
    signal input yesVotes;
    signal input noVotes;
    signal input requiredThreshold;
    signal input timelockPassed;
    signal output valid;
    
    // Calculate net votes
    signal netVotes;
    netVotes <== yesVotes - noVotes;
    
    // Check if net votes exceed threshold
    component voteCheck = GreaterThan(32);
    voteCheck.in[0] <== netVotes;
    voteCheck.in[1] <== noVotes + requiredThreshold;
    voteCheck.out === 1;
    
    // Check if timelock has passed
    component timelockCheck = IsEqual();
    timelockCheck.in[0] <== timelockPassed;
    timelockCheck.in[1] <== 1;
    timelockCheck.out === 1;
    
    valid <== 1;
}

component main {public [proposalId]} = GovernanceAuthCircuit();