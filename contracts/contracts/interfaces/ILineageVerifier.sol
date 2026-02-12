// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ILineageVerifier
 * @notice Interface for the ZK-ORIGIN Lineage Verifier
 */
interface ILineageVerifier {
    /// @notice Emitted when genesis state is set
    event GenesisSet(bytes32 indexed stateHash, bytes32 lineageCommitment);
    
    /// @notice Emitted when a lineage proof is verified
    event LineageVerified(
        bytes32 indexed prevStateHash,
        bytes32 indexed newStateHash,
        bytes32 newLineageCommitment,
        uint256 depth
    );
    
    /// @notice Set the genesis state (can only be called once)
    function setGenesis(
        bytes32 genesisStateHash,
        bytes32 genesisLineageCommitment
    ) external;
    
    /// @notice Verify and record a lineage proof
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[2] calldata publicSignals
    ) external returns (bool);
    
    /// @notice Get the lineage commitment for a state
    function getLineage(bytes32 stateHash) external view returns (bytes32);
    
    /// @notice Check if a state has verified lineage
    function hasVerifiedLineage(bytes32 stateHash) external view returns (bool);
    
    /// @notice Get the current lineage depth for a state
    function getDepth(bytes32 stateHash) external view returns (uint256);
}