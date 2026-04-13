pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/poseidon.circom";
include "../lib/constants.circom";

/*
 * Governance Authentication: Proposal Approval Verification
 * 
 * FIXED: Uses absolute threshold instead of subtraction
 */

template GovernanceAuth() {
    signal input proposalId;
    signal input proposalContentHash;
    signal input transitionHash;
    signal input yesVotes;
    signal input noVotes;
    signal input requiredThreshold;
    signal input proposalTimestamp;
    signal input currentTimestamp;
    
    signal output valid;
    
    // ============ VALIDATE VOTE RANGES ============
    component yesRange = ZKLessThan(32);
    yesRange.in[0] <== yesVotes;
    yesRange.in[1] <== MAX_GOVERNANCE_VOTES();
    yesRange.out === 1;
    
    component noRange = ZKLessThan(32);
    noRange.in[0] <== noVotes;
    noRange.in[1] <== MAX_GOVERNANCE_VOTES();
    noRange.out === 1;
    
    // ============ VERIFY YES VOTES EXCEED THRESHOLD ============
    component voteCheck = ZKGreaterThan(32);
    voteCheck.in[0] <== yesVotes;
    voteCheck.in[1] <== requiredThreshold;
    voteCheck.out === 1;
    
    // ============ VERIFY QUORUM (YES + NO > MINIMUM) ============
    signal totalVotes;
    totalVotes <== yesVotes + noVotes;
    
    component quorumCheck = ZKGreaterThan(32);
    quorumCheck.in[0] <== totalVotes;
    quorumCheck.in[1] <== 0;
    quorumCheck.out === 1;
    
    // ============ VERIFY TIMELOCK EXPIRED ============
    component timeDiffCheck = ZKGreaterEqThan(32);
    timeDiffCheck.in[0] <== currentTimestamp;
    timeDiffCheck.in[1] <== proposalTimestamp;
    timeDiffCheck.out === 1;
    
    signal timeSinceProposal;
    timeSinceProposal <== currentTimestamp - proposalTimestamp;
    
    component timelockCheck = ZKGreaterEqThan(32);
    timelockCheck.in[0] <== timeSinceProposal;
    timelockCheck.in[1] <== GOVERNANCE_TIMELOCK_SECONDS();
    timelockCheck.out === 1;
    
    // ============ VERIFY TRANSITION MATCHES PROPOSAL ============
    component contentMatch = ZKIsEqual();
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