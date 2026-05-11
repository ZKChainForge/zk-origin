// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./LineageVerifier.sol";
import "./interfaces/IAuthorizationVerifier.sol";

contract BatchVerifier {
    
    LineageVerifier public immutable lineageVerifier;
    IAuthorizationVerifier public immutable authVerifier;
    
    struct BatchProof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[20] publicSignals;
        uint8 authType;
        bytes authData;
    }
    
    struct SimpleBatchProof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[20] publicSignals;
    }
    
    struct BatchResult {
        uint256 batchId;
        uint256 proofCount;
        bytes32 finalStateHash;
        bool success;
        uint256 gasUsed;
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
    
    event BatchStatistics(
        uint256 indexed batchId,
        uint256 totalGasUsed,
        uint256 avgGasPerProof,
        uint256 batchDuration
    );
    
    error EmptyBatch();
    error BatchTooLarge();
    error ProofFailed(uint256 index, string reason);
    error ZeroAddress();
    error StateChainBroken(uint256 index);
    error AuthDataMissing(uint256 index);
    error InvalidBatchSequence();
    error DuplicateStateInBatch();
    
    uint256 public totalBatchesProcessed;
    mapping(uint256 => bytes32) public batchFinalStates;
    mapping(uint256 => uint256) public batchProofCounts;
    
    uint256 public constant MAX_BATCH_SIZE = 100;
    
    constructor(
        address _lineageVerifier,
        address _authVerifier
    ) {
        if (_lineageVerifier == address(0)) revert ZeroAddress();
        if (_authVerifier == address(0)) revert ZeroAddress();
        
        lineageVerifier = LineageVerifier(_lineageVerifier);
        authVerifier = IAuthorizationVerifier(_authVerifier);
    }
    
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
            
            bytes32 currentPrevStateHash = bytes32(proof.publicSignals[0]);
            
            if (i > 0 && prevStateHash != currentPrevStateHash) {
                emit BatchFailed(batchId, i, "State chain broken");
                revert StateChainBroken(i);
            }
            
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
                
                bytes32 stateHash = bytes32(proof.publicSignals[1]);
                
                for (uint256 j = 0; j < i; j++) {
                    if (stateHash == bytes32(proofs[j].publicSignals[1])) {
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
        
        bytes32 finalStateHash = bytes32(proofs[proofs.length - 1].publicSignals[1]);
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
            
            bytes32 currentPrevStateHash = bytes32(proof.publicSignals[0]);
            if (i > 0 && prevStateHash != currentPrevStateHash) {
                emit BatchFailed(batchId, i, "State chain broken");
                revert StateChainBroken(i);
            }
            
            bytes memory emptyAuthData = "";
            
            try lineageVerifier.verifyLineage(
                proof.pA,
                proof.pB,
                proof.pC,
                proof.publicSignals,
                0,
                emptyAuthData
            ) returns (bool success) {
                if (!success) {
                    emit BatchFailed(batchId, i, "Proof verification failed");
                    revert ProofFailed(i, "Proof verification failed");
                }
                
                bytes32 stateHash = bytes32(proof.publicSignals[1]);
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
        
        bytes32 finalStateHash = bytes32(proofs[proofs.length - 1].publicSignals[1]);
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
    
    function estimateGas(uint256 proofCount)
        external pure returns (uint256)
    {
        if (proofCount == 0) return 0;
        
        uint256 baseCost = 21000;
        uint256 perProofCost = 250000;
        
        return baseCost + (proofCount * perProofCost);
    }
    
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
    
    function getMaxProofsPerBatch() external pure returns (uint256) {
        return MAX_BATCH_SIZE;
    }
    
    function isValidBatchSize(uint256 proofCount) 
        external pure returns (bool) 
    {
        return proofCount > 0 && proofCount <= MAX_BATCH_SIZE;
    }
    
    function getBatchFinalState(uint256 batchId) 
        external view returns (bytes32) 
    {
        return batchFinalStates[batchId];
    }
    
    function getBatchProofCount(uint256 batchId) 
        external view returns (uint256) 
    {
        return batchProofCounts[batchId];
    }
    
    function getTotalBatches() external view returns (uint256) {
        return totalBatchesProcessed;
    }
}