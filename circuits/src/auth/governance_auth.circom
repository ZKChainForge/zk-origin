pragma circom 2.1.0;

/*
 * Governance Authentication: Proposal Approval Verification
 * 
 * Verifies that:
 * 1. A governance proposal received sufficient votes
 * 2. Timelock has expired
 * 3. The transition matches the proposal content
 */

include "../lib/comparators.circom";
include "../lib/poseidon.circom";
include "../lib/constants.circom";

template GovernanceAuth() {
    // ============ PUBLIC INPUTS ============
    signal input proposalId;
    signal input proposalContentHash;
    signal input transitionHash;
    signal input yesVotes;
    signal input noVotes;
    signal input requiredThreshold;
    signal input proposalTimestamp;
    signal input currentTimestamp;
    
    // ============ OUTPUT ============
    signal output valid;
    
    // ============ VERIFY VOTES EXCEED THRESHOLD ============
    signal netVotes;
    netVotes <== yesVotes - noVotes;
    
    component voteCheck = GreaterThan(32);
    voteCheck.in[0] <== netVotes;
    voteCheck.in[1] <== requiredThreshold;
    voteCheck.out === 1;
    
    // ============ VERIFY TIMELOCK EXPIRED ============
    signal timeSinceProposal;
    timeSinceProposal <== currentTimestamp - proposalTimestamp;
    
    signal timelockDuration;
    timelockDuration <== GOVERNANCE_TIMELOCK_SECONDS();
    
    component timelockCheck = GreaterEqThan(32);
    timelockCheck.in[0] <== timeSinceProposal;
    timelockCheck.in[1] <== timelockDuration;
    timelockCheck.out === 1;
    
    // ============ VERIFY TRANSITION MATCHES PROPOSAL ============
    component contentMatch = IsEqual();
    contentMatch.in[0] <== proposalContentHash;
    contentMatch.in[1] <== transitionHash;
    contentMatch.out === 1;
    
    valid <== 1;
}

component main {public [
    proposalId,
    proposalContentHash,
    transitionHash,
    yesVotes,
    noVotes,
    requiredThreshold,
    proposalTimestamp,
    currentTimestamp
]} = GovernanceAuth();