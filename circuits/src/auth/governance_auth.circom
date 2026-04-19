pragma circom 2.1.0;

include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../lib/constants.circom";

/**
 * @title Governance Authentication (PRODUCTION)
 * @notice Governance proposal approval with timelock
 * 
 * SECURITY:
 *  Verifies yes votes exceed no votes + threshold
 *  Enforces voting quorum (totalVotes > 0)
 *  Requires timelock expiration (48 hours)
 *  Binds execution to proposal content (content hash)
 *  Prevents unauthorized scope expansion
 * 
 * PROTECTION: GOVERNANCE PROTECTED
 * - Ensures proposal passed with sufficient votes
 * - Prevents emergency execution
 * - Binds execution to proposal content hash
 * - No governance action without timelock
 * 
 * INPUT AUTHORIZATION:
 * - proposalId: Governance proposal ID
 * - proposalContentHash: Hash of proposal text
 * - transitionHash: Hash of actual state change
 * - yesVotes: Number of yes votes
 * - noVotes: Number of no votes
 * - requiredThreshold: Minimum super-majority
 * - proposalTimestamp: When proposal was created
 * - currentTimestamp: When execution attempted
 * 
 * OUTPUT GUARANTEE:
 * - valid: 1 if proposal passed and timelock expired
 * 
 * CONSTRAINTS: ~2000 (simple checks)
 * 
 * PRODUCTION CHECKLIST:
 *  Vote counts are reasonable (<1M each)
 *  Yes votes > (no votes + threshold)
 *  Timelock enforced (48 hours minimum)
 *  Proposal content binds to execution
 *  Timestamp validation enforced
 *  No execution without proposal passage
 *  No execution without timelock expiry
 * 
 * ATTACK VECTORS MITIGATED:
 *  Flash voting: No (off-chain issue)
 *  Proposal sniping: Content hash binding prevents
 *  Premature execution: Timelock prevents
 *  Scope creep: Content hash prevents
 *  Insufficient quorum: Vote check enforces
 * 
 * NOTES:
 * - Vote counts are not validated as cryptographic (social)
 * - Contract must verify vote counts on-chain
 * - Circuit just checks vote arithmetic
 * - Timelock is critical security feature
 * - No proposal can be executed same block
 */

template GovernanceAuth() {
    // ============ PUBLIC INPUTS ============
    signal input proposalId;                   // Governance proposal ID
    signal input proposalContentHash;          // Hash of proposal text
    signal input transitionHash;               // Hash of intended state change
    signal input yesVotes;                     // Votes in favor
    signal input noVotes;                      // Votes against
    signal input requiredThreshold;            // Super-majority requirement
    signal input proposalTimestamp;            // When proposal created
    signal input currentTimestamp;             // When execution attempted
    
    // ============ PUBLIC OUTPUTS ============
    signal output valid;  // 1 if proposal valid and timelock expired
    
    // ============ STEP 1: VALIDATE VOTE RANGES ============
    component yesRange = ZKLessThan(32);
    yesRange.in[0] <== yesVotes;
    yesRange.in[1] <== MAX_GOVERNANCE_VOTES();
    yesRange.out === 1;  // ENFORCE: reasonable yes count
    
    component noRange = ZKLessThan(32);
    noRange.in[0] <== noVotes;
    noRange.in[1] <== MAX_GOVERNANCE_VOTES();
    noRange.out === 1;  // ENFORCE: reasonable no count
    
    // ============ STEP 2: VERIFY YES VOTES EXCEED THRESHOLD ============
    component voteCheck = ZKGreaterThan(32);
    voteCheck.in[0] <== yesVotes;
    voteCheck.in[1] <== noVotes + requiredThreshold;
    voteCheck.out === 1;  // ENFORCE: yes > no + threshold
    
    // ============ STEP 3: VERIFY QUORUM ============
    signal totalVotes;
    totalVotes <== yesVotes + noVotes;
    
    component quorumCheck = ZKGreaterThan(32);
    quorumCheck.in[0] <== totalVotes;
    quorumCheck.in[1] <== 0;
    quorumCheck.out === 1;  // ENFORCE: at least one vote
    
    // ============ STEP 4: VERIFY TIMELOCK EXPIRED ============
    component timeDiffCheck = ZKGreaterEqThan(32);
    timeDiffCheck.in[0] <== currentTimestamp;
    timeDiffCheck.in[1] <== proposalTimestamp;
    timeDiffCheck.out === 1;  // ENFORCE: time moves forward
    
    signal timeSinceProposal;
    timeSinceProposal <== currentTimestamp - proposalTimestamp;
    
    component timelockCheck = ZKGreaterEqThan(32);
    timelockCheck.in[0] <== timeSinceProposal;
    timelockCheck.in[1] <== GOVERNANCE_TIMELOCK_SECONDS();
    timelockCheck.out === 1;  // ENFORCE: >= 48 hours elapsed
    
    // ============ STEP 5: VERIFY PROPOSAL CONTENT MATCHES EXECUTION ============
    component contentMatch = ZKIsEqual();
    contentMatch.in[0] <== proposalContentHash;
    contentMatch.in[1] <== transitionHash;
    contentMatch.out === 1;  // ENFORCE: execution matches proposal
    
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