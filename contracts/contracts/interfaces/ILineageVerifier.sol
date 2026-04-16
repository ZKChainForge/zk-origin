// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ILineageVerifier
 * @notice Interface for ZK-ORIGIN state lineage verification
 * 
 * FIXED: Removed duplicate event declarations
 */
interface ILineageVerifier {
    
    // ============ Functions ============
    
    function setGenesis(
        bytes32 genesisStateHash,
        bytes32 genesisLineageCommitment
    ) external;
    
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[19] calldata publicSignals,
        uint8 authType,
        bytes calldata authData
    ) external returns (bool);
    
    function getLineage(bytes32 stateHash) external view returns (bytes32);
    function hasVerifiedLineage(bytes32 stateHash) external view returns (bool);
    function getDepth(bytes32 stateHash) external view returns (uint256);
}