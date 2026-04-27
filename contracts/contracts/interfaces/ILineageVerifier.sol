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
     * @param genesisStateHash Fixed genesis state hash
     * @param genesisLineageCommitment Genesis lineage commitment
     */
    function setGenesis(
        bytes32 genesisStateHash,
        bytes32 genesisLineageCommitment
    ) external;
    
    /**
     * @notice Verify state lineage with authorization
     * @param pA Groth16 proof point A
     * @param pB Groth16 proof point B
     * @param pC Groth16 proof point C
     * @param publicSignals 19 public signals from circuit
     * @param authType Authorization type (0-6)
     * @param authData Encoded authorization data
     * @return true if lineage valid
     */
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[19] calldata publicSignals,
        uint8 authType,
        bytes calldata authData
    ) external returns (bool);
    
    /**
     * @notice Get lineage commitment for state
     * @param stateHash State hash to query
     * @return Lineage commitment
     */
    function getLineage(bytes32 stateHash)
        external view returns (bytes32);
    
    /**
     * @notice Check if state has verified lineage
     * @param stateHash State hash to check
     * @return true if state has verified lineage
     */
    function hasVerifiedLineage(bytes32 stateHash)
        external view returns (bool);
    
    /**
     * @notice Get lineage depth
     * @param stateHash State hash to query
     * @return Depth of lineage chain
     */
    function getDepth(bytes32 stateHash)
        external view returns (uint256);
}