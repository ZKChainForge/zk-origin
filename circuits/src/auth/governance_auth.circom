pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";

/**
 * Governance Authentication
 * 
 * Security:
 * - Vote counts range-checked (prevent overflow tricks)
 * - Yes votes > no votes + threshold
 * - Timelock enforced (48 hours)
 * - Execution bound to proposal via intendedTransitionHash
 *   (proposal stores hash of intended state change, must match)
 * - No hard constraint - parent enforces
 */
template GovernanceAuth() {
    // Public
    signal input proposalId;
    signal input proposalIntendedTransitionHash; // Hash governance approved
    signal input actualTransitionHash;            // Hash being executed NOW
    signal input yesVotes;
    signal input noVotes;
    signal input requiredThreshold;
    signal input proposalTimestamp;
    signal input currentTimestamp;

    // Output - parent enforces === 1
    signal output valid;

    // Step 1: Range check vote counts (prevent field element tricks)
    component yesRange = ZKLessThan(30);  // < 2^30 ~ 1 billion
    yesRange.in[0] <== yesVotes;
    yesRange.in[1] <== 1073741824;
    yesRange.out === 1;

    component noRange = ZKLessThan(30);
    noRange.in[0] <== noVotes;
    noRange.in[1] <== 1073741824;
    noRange.out === 1;

    // Step 2: Yes > no + threshold
    component voteCheck = ZKGreaterThan(32);
    voteCheck.in[0] <== yesVotes;
    voteCheck.in[1] <== noVotes + requiredThreshold;
    voteCheck.out === 1;

    // Step 3: At least one vote
    signal totalVotes;
    totalVotes <== yesVotes + noVotes;
    component quorumCheck = ZKGreaterThan(32);
    quorumCheck.in[0] <== totalVotes;
    quorumCheck.in[1] <== 0;
    quorumCheck.out === 1;

    // Step 4: Timelock - current >= proposal + 48 hours
    component timeOrder = ZKGreaterEqThan(32);
    timeOrder.in[0] <== currentTimestamp;
    timeOrder.in[1] <== proposalTimestamp;
    timeOrder.out === 1;

    signal timeSinceProposal;
    timeSinceProposal <== currentTimestamp - proposalTimestamp;

    component timelockCheck = ZKGreaterEqThan(32);
    timelockCheck.in[0] <== timeSinceProposal;
    timelockCheck.in[1] <== 172800;  // GOVERNANCE_TIMELOCK_SECONDS = 48 hours
    timelockCheck.out === 1;

    // Step 5: Bind execution to proposal
    // Governance must have approved THIS EXACT transition
    component bindingCheck = ZKIsEqual();
    bindingCheck.in[0] <== proposalIntendedTransitionHash;
    bindingCheck.in[1] <== actualTransitionHash;
    bindingCheck.out === 1;

    valid <== 1;
}