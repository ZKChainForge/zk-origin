pragma circom 2.1.0;

include "../lib/comparators.circom";
include "../lib/poseidon.circom";
include "../lib/constants.circom";

/*
 * Governance Authentication: Proposal Approval Verification
 * 
 * SECURITY FIX: Prevents underflow in vote subtraction
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
    
    // ============ VERIFY YES > NO (PREVENTS UNDERFLOW) ============
    component yesGreater = ZKGreaterEqThan(32);
    yesGreater.in[0] <== yesVotes;
    yesGreater.in[1] <== noVotes;
    yesGreater.out === 1;
    
    // ============ COMPUTE NET VOTES (NOW SAFE) ============
    signal netVotes;
    netVotes <== yesVotes - noVotes;
    
    // ============ VERIFY NET VOTES EXCEED THRESHOLD ============
    component voteCheck = ZKGreaterThan(32);
    voteCheck.in[0] <== netVotes;
    voteCheck.in[1] <== requiredThreshold;
    voteCheck.out === 1;
    
    // ============ CHECK NO OVERFLOW IN THRESHOLD CALCULATION ============
    signal thresholdPlusNo;
    thresholdPlusNo <== requiredThreshold + noVotes;
    
    component noOverflow = ZKLessThan(32);
    noOverflow.in[0] <== thresholdPlusNo;
    noOverflow.in[1] <== COUNTER_MAX();
    noOverflow.out === 1;
    
    // ============ VERIFY TIMELOCK EXPIRED ============
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