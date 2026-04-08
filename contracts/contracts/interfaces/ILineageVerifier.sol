// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ILineageVerifier
 * @notice Interface for ZK-ORIGIN state lineage verification
 */
interface ILineageVerifier {
    
    // ============ Events ============
    
    event GenesisSet(
        bytes32 indexed genesisStateHash,
        bytes32 indexed genesisLineageCommitment,
        address indexed admin
    );
    
    event LineageVerified(
        bytes32 indexed prevStateHash,
        bytes32 indexed newStateHash,
        bytes32 indexed newLineageCommitment,
        uint256 depth,
        uint8 originClass,
        uint256 epochId,
        address creator
    );
    
    event RateLimitExceeded(uint8 originClass, uint256 epoch, uint256 current, uint256 limit);
    event OriginPolicyViolated(uint8 fromClass, uint8 toClass);
    event EpochMismatch(uint256 expected, uint256 actual);
    
    // ============ Functions ============
    
    function setGenesis(
        bytes32 genesisStateHash,
        bytes32 genesisLineageCommitment
    ) external;
    
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[12] calldata publicSignals
    ) external returns (bool);
    
    function getLineage(bytes32 stateHash) external view returns (bytes32);
    function hasVerifiedLineage(bytes32 stateHash) external view returns (bool);
    function getDepth(bytes32 stateHash) external view returns (uint256);
}