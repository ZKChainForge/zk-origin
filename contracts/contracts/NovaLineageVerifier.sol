// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./Groth16Verifier.sol";
import "./EpochManager.sol";

/**
 * @title NovaLineageVerifier (PRODUCTION)
 * @notice Verify Nova IVC proofs for ZK-ORIGIN lineage
 * Does NOT implement ILineageVerifier (different interface)
 */

contract NovaLineageVerifier {
    
    // ============ Constants ============
    uint256 public constant VERSION = 3;
    uint256 public constant MAX_DEPTH = 1_000_000;
    
    // Origin classes
    uint8 public constant ORIGIN_GENESIS = 0;
    uint8 public constant ORIGIN_USER = 1;
    uint8 public constant ORIGIN_ADMIN = 2;
    uint8 public constant ORIGIN_BRIDGE = 3;
    uint8 public constant ORIGIN_GOVERNANCE = 4;
    uint8 public constant ORIGIN_SYSTEM = 5;
    uint8 public constant ORIGIN_EMERGENCY = 6;
    
    // ============ Immutable Dependencies ============
    Groth16Verifier public immutable groth16Verifier;
    EpochManager public immutable epochManager;
    
    // ============ Mutable State ============
    address public admin;
    address public pendingAdmin;
    
    bool public genesisInitialized;
    bytes32 public genesisStateHash;
    bytes32 public genesisLineageCommitment;
    bytes32 public currentPolicyRoot;
    bool public isPaused;
    
    // Policy matrix
    bool[7][7] public policyMatrix;
    
    // State tracking
    mapping(bytes32 => bool) public verifiedStates;
    mapping(bytes32 => uint256) public stateTimestamp;
    mapping(bytes32 => address) public stateCreator;
    mapping(bytes32 => uint256) public lineageDepth;
    
    // Prevent proof replay
    mapping(bytes32 => bool) public usedProofs;
    
    // Statistics
    uint256 public totalLineageTransitions;
    uint256 public maxDepthReached;
    
    // ============ Events ============
    
    event NovaProofVerified(
        bytes32 indexed finalStateHash,
        bytes32 indexed genesisStateHash,
        uint256 indexed depth,
        address creator,
        uint256 timestamp
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
    
    event AdminTransferred(address indexed newAdmin);
    event ContractPausedChanged(bool isPausedNow);
    event ProofRejected(bytes32 indexed proofHash, string reason);
    
    // ============ Errors ============
    error NotAdmin();
    error NotPendingAdmin();
    error GenesisAlreadySet();
    error GenesisNotSet();
    error InvalidProof();
    error ProofAlreadyUsed();
    error ContractIsPaused();
    error ZeroAddress();
    error InvalidGenesisState();
    error InvalidDepth();
    error MaxDepthExceeded();
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        require(msg.sender == admin, "NotAdmin");
        _;
    }
    
    modifier whenNotPaused() {
        require(!isPaused, "ContractIsPaused");
        _;
    }
    
    modifier genesisRequired() {
        require(genesisInitialized, "GenesisNotSet");
        _;
    }
    
    // ============ Constructor ============
    
    constructor(
        address _groth16Verifier,
        address _epochManager,
        bytes32 _genesisLineageCommitment,
        bytes32 _policyRoot
    ) {
        require(_groth16Verifier != address(0), "ZeroAddress");
        require(_epochManager != address(0), "ZeroAddress");
        
        groth16Verifier = Groth16Verifier(_groth16Verifier);
        epochManager = EpochManager(_epochManager);
        
        admin = msg.sender;
        currentPolicyRoot = _policyRoot;
        genesisLineageCommitment = _genesisLineageCommitment;
        isPaused = false;
        
        _initializeDefaultPolicy();
    }
    
    // ============ Policy Initialization ============
    
    function _initializeDefaultPolicy() internal {
        policyMatrix[ORIGIN_GENESIS][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_GENESIS][ORIGIN_ADMIN] = true;
        policyMatrix[ORIGIN_GENESIS][ORIGIN_SYSTEM] = true;
        
        policyMatrix[ORIGIN_USER][ORIGIN_USER] = true;
        
        policyMatrix[ORIGIN_ADMIN][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_ADMIN] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_BRIDGE] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_SYSTEM] = true;
        
        policyMatrix[ORIGIN_BRIDGE][ORIGIN_USER] = true;
        
        for (uint8 i = 0; i < 7; i++) {
            policyMatrix[ORIGIN_GOVERNANCE][i] = true;
        }
        
        policyMatrix[ORIGIN_SYSTEM][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_SYSTEM][ORIGIN_SYSTEM] = true;
        
        policyMatrix[ORIGIN_EMERGENCY][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_EMERGENCY][ORIGIN_ADMIN] = true;
        policyMatrix[ORIGIN_EMERGENCY][ORIGIN_SYSTEM] = true;
    }
    
    // ============ Genesis Management ============
    
    function setGenesis(
        bytes32 _genesisStateHash,
        bytes32 _genesisLineageCommitment
    ) external onlyAdmin {
        require(!genesisInitialized, "GenesisAlreadySet");
        require(_genesisStateHash != bytes32(0), "InvalidGenesisState");
        
        genesisStateHash = _genesisStateHash;
        genesisLineageCommitment = _genesisLineageCommitment;
        
        verifiedStates[_genesisStateHash] = true;
        stateTimestamp[_genesisStateHash] = block.timestamp;
        stateCreator[_genesisStateHash] = msg.sender;
        lineageDepth[_genesisStateHash] = 0;
        
        genesisInitialized = true;
        
        emit GenesisSet(_genesisStateHash, _genesisLineageCommitment, msg.sender);
    }
    
    // ============ Nova Proof Verification ============
    
    function verifyNovaProof(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[19] calldata publicSignals
    ) external whenNotPaused genesisRequired returns (bool) {
        
        // Extract signals
        bytes32 genesisState = bytes32(publicSignals[0]);
        bytes32 finalState = bytes32(publicSignals[1]);
        uint256 depth = publicSignals[2];
        
        // Validate inputs
        require(depth > 0, "InvalidDepth");
        require(depth <= MAX_DEPTH, "MaxDepthExceeded");
        require(genesisState == genesisStateHash, "InvalidProof");
        
        // Replay protection
        bytes32 proofHash = keccak256(
            abi.encode(pA, pB, pC, publicSignals)
        );
        require(!usedProofs[proofHash], "ProofAlreadyUsed");
        usedProofs[proofHash] = true;
        
        // Convert signals to uint[12] for Groth16
        uint256[12] memory groth16Signals;
        for (uint256 i = 0; i < 12; i++) {
            groth16Signals[i] = publicSignals[i];
        }
        
        // Verify Groth16 proof
        require(
            groth16Verifier.verifyProof(pA, pB, pC, groth16Signals),
            "InvalidProof"
        );
        
        // Record final state
        if (!verifiedStates[finalState]) {
            verifiedStates[finalState] = true;
            stateTimestamp[finalState] = block.timestamp;
            stateCreator[finalState] = msg.sender;
            lineageDepth[finalState] = depth;
            
            totalLineageTransitions += depth;
            if (depth > maxDepthReached) {
                maxDepthReached = depth;
            }
            
            emit NovaProofVerified(
                finalState,
                genesisState,
                depth,
                msg.sender,
                block.timestamp
            );
        }
        
        return true;
    }
    
    // ============ View Functions ============
    
    function hasVerifiedLineage(bytes32 stateHash)
        external view returns (bool)
    {
        return verifiedStates[stateHash];
    }
    
    function getDepth(bytes32 stateHash)
        external view returns (uint256)
    {
        return lineageDepth[stateHash];
    }
    
    // ============ Admin Functions ============
    
    function updatePolicyRoot(bytes32 _newPolicyRoot) external onlyAdmin {
        currentPolicyRoot = _newPolicyRoot;
    }
    
    function setPolicyTransition(
        uint8 from,
        uint8 to,
        bool allowed
    ) external onlyAdmin {
        require(from < 7 && to < 7, "InvalidClass");
        policyMatrix[from][to] = allowed;
        emit PolicyUpdated(from, to, allowed);
    }
    
    function transferAdmin(address _newAdmin) external onlyAdmin {
        require(_newAdmin != address(0), "ZeroAddress");
        pendingAdmin = _newAdmin;
    }
    
    function acceptAdmin() external {
        require(msg.sender == pendingAdmin, "NotPendingAdmin");
        admin = pendingAdmin;
        pendingAdmin = address(0);
        emit AdminTransferred(admin);
    }
    
    function setPaused(bool _paused) external onlyAdmin {
        isPaused = _paused;
        emit ContractPausedChanged(_paused);
    }
}