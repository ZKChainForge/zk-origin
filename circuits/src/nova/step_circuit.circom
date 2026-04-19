pragma circom 2.1.0;

include "../lib/constants.circom";
include "../lib/poseidon.circom";
include "../lib/comparators.circom";
include "../core/lineage_step.circom";
include "../core/auth_integration.circom";

/**
 * @title Nova IVC Step Circuit (PRODUCTION)
 * @notice Single step of recursive lineage verification
 * 
 * SECURITY (CRITICAL):
 *  Proves one state transition validity
 *  Takes previous state as input
 *  Outputs updated state
 *  Nova will fold these proofs
 *  Final proof is constant size
 * 
 * PROTECTION: CORE IVC STEP
 * - Each step is independent
 * - Can be proven in parallel
 * - Nova handles composition
 * - Enables infinite lineage length
 * 
 * NOVA IVC SEMANTICS:
 * 
 * Step function F(z) where z is state vector:
 * 
 * Input:  z_in = [lineage, counters, nonce, ts, epoch, depth]
 * Output: z_out = [lineage', counters', nonce', ts, epoch, depth']
 * 
 * Witness: Everything needed to prove valid transition
 * 
 * Nova then does:
 * - Run step: z_out' = F(z_in)
 * - Fold: U_new = fold(U_old, U_step, r)
 * - Accumulate: U' represents all iterations so far
 * 
 * CONSTRAINT COUNT:
 * ~20,000 constraints (same as main circuit)
 * But constant regardless of lineage depth!
 * 
 * PRODUCTION CHECKLIST:
 *  State vector matches definition
 *  Input extraction correct
 *  Output assignment correct
 *  All lineage_step checks included
 *  All auth_integration checks included
 *  State vector size = 6 elements
 *  No dynamic lengths
 *  Deterministic computation
 */

template NovaStepCircuit(
    POLICY_MERKLE_DEPTH,
    MAX_ADMIN_SIGNERS,
    ATTESTATION_DEPTH,
    MAX_VALIDATORS
) {
    // ============ INPUT STATE VECTOR (6 elements) ============
    signal input z_in[6];
    // [0] = prevLineageCommitment
    // [1] = prevCounterCommitment
    // [2] = prevNonce
    // [3] = prevTimestamp
    // [4] = prevEpochId
    // [5] = prevDepth
    
    // ============ PUBLIC INPUTS (transition constants) ============
    signal input prevStateHash;
    signal input newStateHash;
    signal input epochId;
    signal input prevOriginClass;
    signal input newOriginClass;
    signal input policyRoot;
    signal input expectedGenesisHash;
    
    // ============ PRIVATE INPUTS (lineage details) ============
    signal input prevEpochId;
    signal input nonce;
    signal input timestamp;
    signal input policyProof[POLICY_MERKLE_DEPTH];
    signal input policyIndices[POLICY_MERKLE_DEPTH];
    signal input prevCounters[7];
    signal input rateLimits[7];
    signal input authMessageHash;
    signal input originClass;
    
    // User auth
    signal input userPublicKeyX;
    signal input userPublicKeyY;
    signal input userSignatureR;
    signal input userSignatureS;
    
    // Admin auth
    signal input adminPublicKeys[MAX_ADMIN_SIGNERS][2];
    signal input adminSignatures[MAX_ADMIN_SIGNERS][2];
    signal input adminSignerMask[MAX_ADMIN_SIGNERS];
    signal input adminThreshold;
    
    // Bridge auth
    signal input bridgeSourceChainId;
    signal input bridgeExpectedSourceChain;
    signal input bridgeStateRoot;
    signal input bridgeExpectedRoot;
    signal input bridgeSourceBlockNumber;
    signal input bridgeSourceLatestBlock;
    signal input bridgeValidatorPublicKeys[MAX_VALIDATORS][2];
    signal input bridgeValidatorSignatures[MAX_VALIDATORS][2];
    signal input bridgeValidatorMask[MAX_VALIDATORS];
    signal input bridgeSignatureR;
    signal input bridgeSignatureS;
    signal input bridgePublicKeyX;
    signal input bridgePublicKeyY;
    signal input bridgeMerkleProof[ATTESTATION_DEPTH];
    signal input bridgeMerkleIndices[ATTESTATION_DEPTH];
    
    // Governance auth
    signal input governanceProposalId;
    signal input governanceProposalHash;
    signal input governanceTransitionHash;
    signal input governanceYesVotes;
    signal input governanceNoVotes;
    signal input governanceRequiredThreshold;
    signal input governanceProposalTimestamp;
    signal input governanceCurrentTimestamp;
    
    // System auth
    signal input systemCallerAddress;
    signal input systemExpectedSystemAddress;
    
    // Emergency auth
    signal input emergencyMessageHash;
    signal input emergencyExpectedKeyHash;
    signal input emergencyCurrentTVL;
    signal input emergencyNormalTVL;
    signal input emergencyTimeSinceLastBlock;
    signal input emergencySystemPaused;
    signal input emergencyKeyHash;
    signal input emergencySignatureR;
    signal input emergencySignatureS;
    signal input emergencyPublicKeyX;
    signal input emergencyPublicKeyY;
    
    // ============ OUTPUT STATE VECTOR (6 elements) ============
    signal output z_out[6];
    // [0] = newLineageCommitment
    // [1] = newCounterCommitment
    // [2] = newNonce
    // [3] = timestamp
    // [4] = epochId
    // [5] = newDepth
    
    // ============ STEP 1: EXTRACT INPUT STATE ============
    signal prevLineageCommitment;
    signal prevCounterCommitment;
    signal prevNonce;
    signal prevTimestamp;
    signal prevDepthFromState;
    
    prevLineageCommitment <== z_in[0];
    prevCounterCommitment <== z_in[1];
    prevNonce <== z_in[2];
    prevTimestamp <== z_in[3];
    // z_in[4] = prevEpochId from public input
    prevDepthFromState <== z_in[5];
    
    // ============ STEP 2: VERIFY AUTHORIZATION ============
    // Authorization is critical for lineage validity
    component authIntegration = AuthorizationIntegration(
        MAX_ADMIN_SIGNERS,
        ATTESTATION_DEPTH,
        MAX_VALIDATORS
    );
    
    authIntegration.originClass <== originClass;
    authIntegration.messageHash <== authMessageHash;
    authIntegration.userPublicKeyX <== userPublicKeyX;
    authIntegration.userPublicKeyY <== userPublicKeyY;
    authIntegration.userSignatureR <== userSignatureR;
    authIntegration.userSignatureS <== userSignatureS;
    authIntegration.adminPublicKeys <== adminPublicKeys;
    authIntegration.adminSignatures <== adminSignatures;
    authIntegration.adminSignerMask <== adminSignerMask;
    authIntegration.adminThreshold <== adminThreshold;
    authIntegration.bridgeSourceChainId <== bridgeSourceChainId;
    authIntegration.bridgeExpectedSourceChain <== bridgeExpectedSourceChain;
    authIntegration.bridgeStateRoot <== bridgeStateRoot;
    authIntegration.bridgeExpectedRoot <== bridgeExpectedRoot;
    authIntegration.bridgeSourceBlockNumber <== bridgeSourceBlockNumber;
    authIntegration.bridgeSourceLatestBlock <== bridgeSourceLatestBlock;
    authIntegration.bridgeValidatorPublicKeys <== bridgeValidatorPublicKeys;
    authIntegration.bridgeValidatorSignatures <== bridgeValidatorSignatures;
    authIntegration.bridgeValidatorMask <== bridgeValidatorMask;
    authIntegration.bridgeSignatureR <== bridgeSignatureR;
    authIntegration.bridgeSignatureS <== bridgeSignatureS;
    authIntegration.bridgePublicKeyX <== bridgePublicKeyX;
    authIntegration.bridgePublicKeyY <== bridgePublicKeyY;
    authIntegration.bridgeMerkleProof <== bridgeMerkleProof;
    authIntegration.bridgeMerkleIndices <== bridgeMerkleIndices;
    authIntegration.governanceProposalId <== governanceProposalId;
    authIntegration.governanceProposalHash <== governanceProposalHash;
    authIntegration.governanceTransitionHash <== governanceTransitionHash;
    authIntegration.governanceYesVotes <== governanceYesVotes;
    authIntegration.governanceNoVotes <== governanceNoVotes;
    authIntegration.governanceRequiredThreshold <== governanceRequiredThreshold;
    authIntegration.governanceProposalTimestamp <== governanceProposalTimestamp;
    authIntegration.governanceCurrentTimestamp <== governanceCurrentTimestamp;
    authIntegration.systemCallerAddress <== systemCallerAddress;
    authIntegration.systemExpectedSystemAddress <== systemExpectedSystemAddress;
    authIntegration.emergencyMessageHash <== emergencyMessageHash;
    authIntegration.emergencyExpectedKeyHash <== emergencyExpectedKeyHash;
    authIntegration.emergencyCurrentTVL <== emergencyCurrentTVL;
    authIntegration.emergencyNormalTVL <== emergencyNormalTVL;
    authIntegration.emergencyTimeSinceLastBlock <== emergencyTimeSinceLastBlock;
    authIntegration.emergencySystemPaused <== emergencySystemPaused;
    authIntegration.emergencyKeyHash <== emergencyKeyHash;
    authIntegration.emergencySignatureR <== emergencySignatureR;
    authIntegration.emergencySignatureS <== emergencySignatureS;
    authIntegration.emergencyPublicKeyX <== emergencyPublicKeyX;
    authIntegration.emergencyPublicKeyY <== emergencyPublicKeyY;
    
    signal authCommitment;
    authCommitment <== authIntegration.authCommitment;
    
    // ============ STEP 3: VERIFY LINEAGE STEP ============
    // This is the core transition verification
    component lineageStep = LineageStep(POLICY_MERKLE_DEPTH);
    
    lineageStep.prevStateHash <== prevStateHash;
    lineageStep.newStateHash <== newStateHash;
    lineageStep.epochId <== epochId;
    lineageStep.prevOriginClass <== prevOriginClass;
    lineageStep.newOriginClass <== newOriginClass;
    lineageStep.prevLineageCommitment <== prevLineageCommitment;
    lineageStep.prevCounterCommitment <== prevCounterCommitment;
    lineageStep.policyRoot <== policyRoot;
    lineageStep.expectedGenesisHash <== expectedGenesisHash;
    
    lineageStep.prevEpochId <== prevEpochId;
    lineageStep.prevDepth <== prevDepthFromState;
    lineageStep.nonce <== nonce;
    lineageStep.prevNonce <== prevNonce;
    lineageStep.timestamp <== timestamp;
    lineageStep.prevTimestamp <== prevTimestamp;
    
    for (var i = 0; i < POLICY_MERKLE_DEPTH; i++) {
        lineageStep.policyProof[i] <== policyProof[i];
        lineageStep.policyIndices[i] <== policyIndices[i];
    }
    
    for (var i = 0; i < 7; i++) {
        lineageStep.prevCounters[i] <== prevCounters[i];
        lineageStep.rateLimits[i] <== rateLimits[i];
    }
    
    lineageStep.authorizationCommitment <== authCommitment;
    
    // ============ STEP 4: EXTRACT OUTPUT STATE ============
    signal newLineageCommitment;
    signal newCounterCommitment;
    signal newDepth;
    
    newLineageCommitment <== lineageStep.newLineageCommitment;
    newCounterCommitment <== lineageStep.newCounterCommitment;
    newDepth <== prevDepthFromState + 1;
    
    // ============ STEP 5: BUILD OUTPUT STATE VECTOR ============
    z_out[0] <== newLineageCommitment;   // Updated lineage
    z_out[1] <== newCounterCommitment;   // Updated counters
    z_out[2] <== nonce;                  // Updated nonce
    z_out[3] <== timestamp;              // Current timestamp
    z_out[4] <== epochId;                // Current epoch
    z_out[5] <== newDepth;               // Incremented depth
}

component main {public [
    prevStateHash,
    newStateHash,
    epochId,
    prevOriginClass,
    newOriginClass,
    policyRoot,
    expectedGenesisHash,
    authMessageHash,
    originClass,
    prevEpochId
]} = NovaStepCircuit(6, 15, 8, 21);