// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ILineageVerifier
 * @notice Interface for state lineage verification
 */

interface ILineageVerifier {
    
    // ============ Events ============
    
    event LineageVerified(
        bytes32 indexed prevStateHash,
        bytes32 indexed newStateHash,
        bytes32 indexed newLineageCommitment,
        uint256 depth,
        uint8 originClass,
        uint256 epochId,
        address creator,
        bytes32 authorizationCommitment
    );
    
    event GenesisSet(
        bytes32 indexed genesisStateHash,
        bytes32 indexed genesisLineageCommitment,
        address indexed admin
    );
    
    event PolicyUpdated(
        uint8 indexed fromClass,
        uint8 indexed toClass,
        bool allowed
    );
    
    event EpochTransition(
        uint256 indexed oldEpoch,
        uint256 indexed newEpoch,
        uint256 timestamp
    );
    
    event ProofRejected(
        bytes32 indexed proofHash,
        string reason
    );
    
    // ============ Core Functions ============
    
    /**
     * @notice Set genesis state (immutable, one-time)
     */
    function setGenesis(
        bytes32 genesisStateHash,
        bytes32 genesisLineageCommitment
    ) external;
    
    /**
     * @notice Get lineage commitment for state
     */
    function getLineage(bytes32 stateHash)
        external view returns (bytes32);
    
    /**
     * @notice Check if state has verified lineage
     */
    function hasVerifiedLineage(bytes32 stateHash)
        external view returns (bool);
    
    /**
     * @notice Get lineage depth
     */
    function getDepth(bytes32 stateHash)
        external view returns (uint256);
}