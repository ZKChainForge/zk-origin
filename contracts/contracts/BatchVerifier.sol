// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./LineageVerifier.sol";
import "./interfaces/IAuthorizationVerifier.sol";

/**
 * @title BatchVerifier
 * @notice Verify multiple lineage proofs in a single transaction
 * 
 * UPDATED: Handles new verifyLineage signature with 6 parameters
 */
contract BatchVerifier {
    
    LineageVerifier public immutable lineageVerifier;
    IAuthorizationVerifier public immutable authVerifier;
    
    struct BatchProof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[19] publicSignals;  // Updated: was uint[12], now uint[19]
        uint8 authType;
        bytes authData;
    }
    
    struct SimpleBatchProof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[19] publicSignals;
    }
    
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
    
    error EmptyBatch();
    error BatchTooLarge();
    error ProofFailed(uint256 index, string reason);
    error ZeroAddress();
    error StateChainBroken(uint256 index);
    error AuthDataMissing(uint256 index);
    
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
     * Each proof must include authType and authData
     */
    function verifyBatchWithAuth(BatchProof[] calldata proofs)
        external
        returns (bool)
    {
        if (proofs.length == 0) revert EmptyBatch();
        if (proofs.length > 100) revert BatchTooLarge();
        
        bytes32 prevStateHash = bytes32(0);
        
        for (uint256 i = 0; i < proofs.length; i++) {
            BatchProof calldata proof = proofs[i];
            
            // Check continuity (except for first proof)
            bytes32 currentPrevStateHash = bytes32(proof.publicSignals[3]);
            if (i > 0 && prevStateHash != currentPrevStateHash) {
                emit BatchFailed(block.number, i, "State chain broken");
                revert StateChainBroken(i);
            }
            
            // Verify authorization data is provided
            if (proof.authData.length == 0) {
                emit BatchFailed(block.number, i, "Missing auth data");
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
                    emit BatchFailed(block.number, i, "Proof verification failed");
                    revert ProofFailed(i, "Proof verification failed");
                }
                
                bytes32 stateHash = bytes32(proof.publicSignals[4]);
                emit ProofVerified(i, stateHash, msg.sender);
                prevStateHash = stateHash;
                
            } catch Error(string memory reason) {
                emit BatchFailed(block.number, i, reason);
                revert ProofFailed(i, reason);
            } catch (bytes memory) {
                emit BatchFailed(block.number, i, "Unknown error");
                revert ProofFailed(i, "Unknown error");
            }
        }
        
        bytes32 finalStateHash = bytes32(
            proofs[proofs.length - 1].publicSignals[4]
        );
        
        emit BatchVerified(
            block.number,
            proofs.length,
            finalStateHash,
            true
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
        if (proofs.length > 100) revert BatchTooLarge();
        
        bytes32 prevStateHash = bytes32(0);
        
        for (uint256 i = 0; i < proofs.length; i++) {
            SimpleBatchProof calldata proof = proofs[i];
            
            // Check continuity
            bytes32 currentPrevStateHash = bytes32(proof.publicSignals[3]);
            if (i > 0 && prevStateHash != currentPrevStateHash) {
                emit BatchFailed(block.number, i, "State chain broken");
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
                    emit BatchFailed(block.number, i, "Proof verification failed");
                    revert ProofFailed(i, "Proof verification failed");
                }
                
                bytes32 stateHash = bytes32(proof.publicSignals[4]);
                emit ProofVerified(i, stateHash, msg.sender);
                prevStateHash = stateHash;
                
            } catch Error(string memory reason) {
                emit BatchFailed(block.number, i, reason);
                revert ProofFailed(i, reason);
            } catch (bytes memory) {
                emit BatchFailed(block.number, i, "Unknown error");
                revert ProofFailed(i, "Unknown error");
            }
        }
        
        bytes32 finalStateHash = bytes32(
            proofs[proofs.length - 1].publicSignals[4]
        );
        
        emit BatchVerified(
            block.number,
            proofs.length,
            finalStateHash,
            true
        );
        
        return true;
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Estimate gas for batch
     * 
     * Rough estimate: 21000 base + 250000 per proof
     * Actual may vary based on authorization type and data
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
        return 100;
    }
    
    /**
     * @notice Check if batch size is valid
     */
    function isValidBatchSize(uint256 proofCount) external pure returns (bool) {
        return proofCount > 0 && proofCount <= 100;
    }
}