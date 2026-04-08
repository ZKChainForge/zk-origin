// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./LineageVerifier.sol";

/**
 * @title BatchVerifier
 * @notice Verify multiple lineage proofs in a single transaction
 */
contract BatchVerifier {
    
    LineageVerifier public immutable lineageVerifier;
    
    struct BatchProof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[12] publicSignals;
    }
    
    event BatchVerified(
        uint256 indexed batchId,
        uint256 proofCount,
        bytes32 finalStateHash,
        bool success
    );
    
    error EmptyBatch();
    error BatchTooLarge();
    error ProofFailed(uint256 index);
    error ZeroAddress();
    
    constructor(address _lineageVerifier) {
        if (_lineageVerifier == address(0)) revert ZeroAddress();
        lineageVerifier = LineageVerifier(_lineageVerifier);
    }
    
    /**
     * @notice Verify a batch of proofs
     * @param proofs Array of proofs
     * @return success Whether all proofs verified
     */
    function verifyBatch(BatchProof[] calldata proofs)
        external
        returns (bool)
    {
        if (proofs.length == 0) revert EmptyBatch();
        if (proofs.length > 100) revert BatchTooLarge();
        
        for (uint256 i = 0; i < proofs.length; i++) {
            bool success = lineageVerifier.verifyLineage(
                proofs[i].pA,
                proofs[i].pB,
                proofs[i].pC,
                proofs[i].publicSignals
            );
            if (!success) revert ProofFailed(i);
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
    
    /**
     * @notice Estimate gas for batch
     */
    function estimateGas(uint256 proofCount)
        external
        pure
        returns (uint256)
    {
        return 21000 + (proofCount * 250000);
    }
}