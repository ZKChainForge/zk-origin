// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./LineageVerifier.sol";

contract BatchVerifier {
    LineageVerifier public immutable lineageVerifier;
    
    struct ProofData {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[5] publicSignals;
    }
    
    event BatchVerified(uint256 indexed batchId, uint256 count, bytes32 batchRoot);
    
    error EmptyBatch();
    error BatchTooLarge();
    error ProofFailed(uint256 index);
    error FinalLineageMismatch();
    
    constructor(address _lineageVerifier) {
        require(_lineageVerifier != address(0), "Zero address");
        lineageVerifier = LineageVerifier(_lineageVerifier);
    }
    
    /**
     * @notice Verify a batch of proofs using the full 5-signal verification
     * @param proofs Array of proof data
     * @param expectedFinalLineage Expected lineage commitment of the final state
     */
    function verifyBatch(
        ProofData[] calldata proofs,
        bytes32 expectedFinalLineage
    ) external returns (bool) {
        if (proofs.length == 0) revert EmptyBatch();
        if (proofs.length > 100) revert BatchTooLarge();
        
        for (uint256 i = 0; i < proofs.length; i++) {
            // Use verifyLineageFull which accepts 5 public signals
            bool success = lineageVerifier.verifyLineageFull(
                proofs[i].pA,
                proofs[i].pB,
                proofs[i].pC,
                proofs[i].publicSignals
            );
            if (!success) revert ProofFailed(i);
        }
        
        // Verify final state matches expected
        bytes32 finalState = bytes32(proofs[proofs.length - 1].publicSignals[4]);
        bytes32 finalLineage = lineageVerifier.stateLineage(finalState);
        if (finalLineage != expectedFinalLineage) revert FinalLineageMismatch();
        
        emit BatchVerified(block.number, proofs.length, expectedFinalLineage);
        return true;
    }
    
    /**
     * @notice Estimate gas for batch verification
     * @param proofCount Number of proofs in the batch
     */
    function estimateBatchGas(uint256 proofCount) external pure returns (uint256) {
        // Base cost + per-proof verification cost
        return 21000 + (proofCount * 250000);
    }
}