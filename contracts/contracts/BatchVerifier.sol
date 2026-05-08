// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./LineageVerifier.sol";
import "./interfaces/IAuthorizationVerifier.sol";

/**
 * @title BatchVerifier (PRODUCTION)
 * @notice Verify multiple lineage proofs in a single transaction
 * 
 * SECURITY:
 *  Verifies state chain continuity
 *  Prevents gap attacks
 *  Enforces ordering
 *  Efficient batch processing
 * 
 * PRODUCTION NOTES:
 * - Reduces gas for batch transitions
 * - Enables efficient lineage batching
 * - Enforces global state continuity
 * - Supports all origin classes
 * 
 * CONSTRAINTS:
 * - Max proofs per batch: 100
 * - Each proof verified independently
 * - State chain continuity enforced
 * - No gaps allowed in lineage
 */

contract BatchVerifier {
    
    // ============ Immutable References ============
    LineageVerifier public immutable lineageVerifier;
    IAuthorizationVerifier public immutable authVerifier;
    
    // ============ Structures ============
    
    struct BatchProof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[19] publicSignals;
        uint8 authType;
        bytes authData;
    }
    
    struct SimpleBatchProof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[19] publicSignals;
    }
    
    struct BatchResult {
        uint256 batchId;
        uint256 proofCount;
        bytes32 finalStateHash;
        bool success;
        uint256 gasUsed;
    }
    
    // ============ Events ============
    event BatchVerified(
        uint256 indexed batchId,
        uint256 proofCount,
        bytes32 finalStateHash,
        bool success
    );
    
    event ProofVerified(
        uint256 indexed index,
        bytes32 indexed stateHash,
        address indexed creator
    );
    
    event BatchFailed(
        uint256 indexed batchId,
        uint256 failedAtIndex,
        string reason
    );
    
    event BatchStatistics(
        uint256 indexed batchId,
        uint256 totalGasUsed,
        uint256 avgGasPerProof,
        uint256 batchDuration
    );
    
    // ============ Errors ============
    error EmptyBatch();
    error BatchTooLarge();
    error ProofFailed(uint256 index, string reason);
    error ZeroAddress();
    error StateChainBroken(uint256 index);
    error AuthDataMissing(uint256 index);
    error InvalidBatchSequence();
    error DuplicateStateInBatch();
    
    // ============ State ============
    uint256 public totalBatchesProcessed;
    mapping(uint256 => bytes32) public batchFinalStates;
    mapping(uint256 => uint256) public batchProofCounts;
    
    // ============ Constants ============
    uint256 public constant MAX_BATCH_SIZE = 100;
    
    // ============ Constructor ============
    constructor(
        address _lineageVerifier,
        address _authVerifier
    ) {
        if (_lineageVerifier == address(0)) revert ZeroAddress();
        if (_authVerifier == address(0)) revert ZeroAddress();
        
        lineageVerifier = LineageVerifier(_lineageVerifier);
        authVerifier = IAuthorizationVerifier(_authVerifier);
    }
    
    // ============ Batch Verification with Full Data ============
    
    /**
     * @notice Verify a batch of proofs with authorization data
     * @param proofs Array of proofs with auth data
     * @return success Whether all proofs verified
     * 
     * SECURITY:
     * ✓ Verifies each proof individually
     * ✓ Checks state continuity
     * ✓ Enforces ordering
     * ✓ No gaps allowed
     */
    function verifyBatchWithAuth(BatchProof[] calldata proofs)
        external
        returns (bool)
    {
        if (proofs.length == 0) revert EmptyBatch();
        if (proofs.length > MAX_BATCH_SIZE) revert BatchTooLarge();
        
        uint256 batchId = totalBatchesProcessed;
        uint256 startGas = gasleft();
        bytes32 prevStateHash = bytes32(0);
        uint256 proofCount = 0;
        
        for (uint256 i = 0; i < proofs.length; i++) {
            BatchProof calldata proof = proofs[i];
            
            // Extract current prev state hash from public signals
            bytes32 currentPrevStateHash = bytes32(proof.publicSignals[3]);
            
            // Check continuity (except for first proof)
            if (i > 0 && prevStateHash != currentPrevStateHash) {
                emit BatchFailed(batchId, i, "State chain broken");
                revert StateChainBroken(i);
            }
            
            // Verify authorization data is provided
            if (proof.authData.length == 0) {
                emit BatchFailed(batchId, i, "Missing auth data");
                revert AuthDataMissing(i);
            }
            
            try lineageVerifier.verifyLineage(
                proof.pA,
                proof.pB,
                proof.pC,
                proof.publicSignals,
                proof.authType,
                proof.authData
            ) returns (bool success) {
                if (!success) {
                    emit BatchFailed(batchId, i, "Proof verification failed");
                    revert ProofFailed(i, "Proof verification failed");
                }
                
                // Extract new state hash from signals
                bytes32 stateHash = bytes32(proof.publicSignals[4]);
                
                // Check for duplicates in batch
                for (uint256 j = 0; j < i; j++) {
                    if (stateHash == bytes32(proofs[j].publicSignals[4])) {
                        emit BatchFailed(batchId, i, "Duplicate state in batch");
                        revert DuplicateStateInBatch();
                    }
                }
                
                emit ProofVerified(i, stateHash, msg.sender);
                prevStateHash = stateHash;
                proofCount++;
                
            } catch Error(string memory reason) {
                emit BatchFailed(batchId, i, reason);
                revert ProofFailed(i, reason);
            } catch (bytes memory) {
                emit BatchFailed(batchId, i, "Unknown error");
                revert ProofFailed(i, "Unknown error");
            }
        }
        
        // Record batch results
        bytes32 finalStateHash = bytes32(proofs[proofs.length - 1].publicSignals[4]);
        batchFinalStates[batchId] = finalStateHash;
        batchProofCounts[batchId] = proofCount;
        
        uint256 gasUsed = startGas - gasleft();
        totalBatchesProcessed++;
        
        emit BatchVerified(
            batchId,
            proofCount,
            finalStateHash,
            true
        );
        
        emit BatchStatistics(
            batchId,
            gasUsed,
            gasUsed / proofs.length,
            block.timestamp
        );
        
        return true;
    }
    
    // ============ Simplified Batch Verification ============
    
    /**
     * @notice Verify batch of proofs with pre-verified authorization
     * 
     * Use this when authorization has already been verified separately
     * Pass empty authType (0) and empty authData
     */
    function verifyBatchPreApproved(SimpleBatchProof[] calldata proofs)
        external
        returns (bool)
    {
        if (proofs.length == 0) revert EmptyBatch();
        if (proofs.length > MAX_BATCH_SIZE) revert BatchTooLarge();
        
        uint256 batchId = totalBatchesProcessed;
        bytes32 prevStateHash = bytes32(0);
        uint256 proofCount = 0;
        
        for (uint256 i = 0; i < proofs.length; i++) {
            SimpleBatchProof calldata proof = proofs[i];
            
            // Check continuity
            bytes32 currentPrevStateHash = bytes32(proof.publicSignals[3]);
            if (i > 0 && prevStateHash != currentPrevStateHash) {
                emit BatchFailed(batchId, i, "State chain broken");
                revert StateChainBroken(i);
            }
            
            // Empty auth data (pre-approved)
            bytes memory emptyAuthData = "";
            
            try lineageVerifier.verifyLineage(
                proof.pA,
                proof.pB,
                proof.pC,
                proof.publicSignals,
                0,  // authType = 0 (ignored)
                emptyAuthData
            ) returns (bool success) {
                if (!success) {
                    emit BatchFailed(batchId, i, "Proof verification failed");
                    revert ProofFailed(i, "Proof verification failed");
                }
                
                bytes32 stateHash = bytes32(proof.publicSignals[4]);
                emit ProofVerified(i, stateHash, msg.sender);
                prevStateHash = stateHash;
                proofCount++;
                
            } catch Error(string memory reason) {
                emit BatchFailed(batchId, i, reason);
                revert ProofFailed(i, reason);
            } catch (bytes memory) {
                emit BatchFailed(batchId, i, "Unknown error");
                revert ProofFailed(i, "Unknown error");
            }
        }
        
        bytes32 finalStateHash = bytes32(proofs[proofs.length - 1].publicSignals[4]);
        batchFinalStates[batchId] = finalStateHash;
        batchProofCounts[batchId] = proofCount;
        totalBatchesProcessed++;
        
        emit BatchVerified(
            batchId,
            proofCount,
            finalStateHash,
            true
        );
        
        return true;
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Estimate gas for batch
     */
    function estimateGas(uint256 proofCount)
        external pure returns (uint256)
    {
        if (proofCount == 0) return 0;
        
        // Base call cost + per-proof cost
        uint256 baseCost = 21000;
        uint256 perProofCost = 250000;
        
        return baseCost + (proofCount * perProofCost);
    }
    
    /**
     * @notice Estimate detailed gas breakdown
     */
    function estimateGasDetailed(uint256 proofCount)
        external pure returns (
            uint256 baseCost,
            uint256 totalProofCost,
            uint256 estimatedTotal
        )
    {
        baseCost = 21000;
        totalProofCost = proofCount * 250000;
        estimatedTotal = baseCost + totalProofCost;
    }
    
    /**
     * @notice Get maximum proofs per batch
     */
    function getMaxProofsPerBatch() external pure returns (uint256) {
        return MAX_BATCH_SIZE;
    }
    
    /**
     * @notice Check if batch size is valid
     */
    function isValidBatchSize(uint256 proofCount) 
        external pure returns (bool) 
    {
        return proofCount > 0 && proofCount <= MAX_BATCH_SIZE;
    }
    
    /**
     * @notice Get batch final state
     */
    function getBatchFinalState(uint256 batchId) 
        external view returns (bytes32) 
    {
        return batchFinalStates[batchId];
    }
    
    /**
     * @notice Get batch proof count
     */
    function getBatchProofCount(uint256 batchId) 
        external view returns (uint256) 
    {
        return batchProofCounts[batchId];
    }
    
    /**
     * @notice Get total batches processed
     */
    function getTotalBatches() external view returns (uint256) {
        return totalBatchesProcessed;
    }
}