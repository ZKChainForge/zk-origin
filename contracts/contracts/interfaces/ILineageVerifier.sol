// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface ILineageVerifier {
    
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
    
    function setGenesis(
        bytes32 genesisStateHash,
        bytes32 genesisLineageCommitment
    ) external;
    
    //  FIXED: Changed from uint256[19] to uint256[20]
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[20] calldata publicSignals,  //  CHANGED: 19 → 20
        uint8 authType,
        bytes calldata authData
    ) external returns (bool);
    
    function getLineage(bytes32 stateHash)
        external view returns (bytes32);
    
    function hasVerifiedLineage(bytes32 stateHash)
        external view returns (bool);
    
    function getDepth(bytes32 stateHash)
        external view returns (uint256);
}