// contracts/contracts/BatchVerifier.sol
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
    
    constructor(address _lineageVerifier) {
        lineageVerifier = LineageVerifier(_lineageVerifier);
    }
    
    function verifyBatch(
        ProofData[] calldata proofs,
        bytes32 expectedFinalLineage
    ) external returns (bool) {
        require(proofs.length > 0, "Empty batch");
        require(proofs.length <= 100, "Batch too large");
        
        for (uint256 i = 0; i < proofs.length; i++) {
            bool success = lineageVerifier.verifyLineage(
                proofs[i].pA,
                proofs[i].pB,
                proofs[i].pC,
                proofs[i].publicSignals
            );
            require(success, string(abi.encodePacked("Proof ", i, " failed")));
        }
        
        // Verify final state matches expected
        bytes32 finalState = bytes32(proofs[proofs.length - 1].publicSignals[4]);
        bytes32 finalLineage = lineageVerifier.stateLineage(finalState);
        require(finalLineage == expectedFinalLineage, "Final lineage mismatch");
        
        emit BatchVerified(block.number, proofs.length, expectedFinalLineage);
        return true;
    }
    
    function estimateBatchGas(uint256 proofCount) external pure returns (uint256) {
        // Base cost + per-proof verification cost
        return 21000 + (proofCount * 250000);
    }
}