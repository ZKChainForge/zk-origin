// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/ILineageVerifier.sol";
import "./Groth16Verifier.sol";
import "./EpochManager.sol";
import "./RateLimiter.sol";

/**
 * @title LineageVerifier
 * @notice Core contract for ZK-ORIGIN state lineage verification
 * @dev Verifies state transitions using Groth16 zero-knowledge proofs
 */
contract LineageVerifier is ILineageVerifier {
    
    // ============ Constants ============
    
    uint256 public constant MAX_DEPTH = 1_000_000;
    uint256 public constant VERSION = 1;
    
    // Origin classes (must match circuit)
    uint8 public constant ORIGIN_GENESIS = 0;
    uint8 public constant ORIGIN_USER = 1;
    uint8 public constant ORIGIN_ADMIN = 2;
    uint8 public constant ORIGIN_BRIDGE = 3;
    uint8 public constant ORIGIN_GOVERNANCE = 4;
    uint8 public constant ORIGIN_SYSTEM = 5;
    uint8 public constant ORIGIN_EMERGENCY = 6;
    
    // ============ Immutable State ============
    
    Groth16Verifier public immutable groth16Verifier;
    EpochManager public immutable epochManager;
    RateLimiter public immutable rateLimiter;
    
    // ============ Mutable State ============
    
    address public admin;
    address public pendingAdmin;
    
    bool public genesisInitialized;
    bytes32 public genesisStateHash;
    bytes32 public genesisLineageCommitment;
    
    bytes32 public currentPolicyRoot;
    bool public paused;
    bool public allowDuplicateStates;
    
    mapping(bytes32 => bytes32) public stateLineage;
    mapping(bytes32 => uint256) public stateDepth;
    mapping(bytes32 => bool) public verifiedStates;
    mapping(bytes32 => uint8) public stateOriginClass;
    mapping(bytes32 => uint256) public stateTimestamp;
    mapping(bytes32 => address) public stateCreator;
    mapping(bytes32 => bool) public usedProofs;
    
    uint256 public totalTransitions;
    uint256 public maxDepthReached;
    
    // ============ Structs ============
    
    struct PublicSignals {
        bytes32 prevLineageCommitment;    // Index 0
        bytes32 newLineageCommitment;     // Index 1
        bytes32 policyRoot;               // Index 2
        bytes32 prevStateHash;            // Index 3
        bytes32 newStateHash;             // Index 4
        bytes32 prevCounterCommitment;    // Index 5
        bytes32 newCounterCommitment;     // Index 6
        uint256 epochId;                  // Index 7
        uint256 prevOriginClass;          // Index 8
        uint256 newOriginClass;           // Index 9
        uint256 timestamp;                // Index 10
        uint256 lineageValid;             // Index 11
    }
    
    // ============ Errors ============
    
    error NotAdmin();
    error NotPendingAdmin();
    error GenesisAlreadySet();
    error GenesisNotSet();
    error InvalidProof();
    error PreviousStateNotVerified();
    error LineageMismatch();
    error PolicyMismatch();
    error ZeroStateHash();
    error ZeroAddress();
    error MaxDepthExceeded();
    error ContractPaused();
    error ProofAlreadyUsed();
    error StateAlreadyExists();
    error InvalidOriginClass();
    error EpochMismatch();
    error RateLimitExceeded();
    error InvalidLineageValid();
    
    // ============ Modifiers ============
    
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    modifier whenNotPaused() {
        if (paused) revert ContractPaused();
        _;
    }
    
    modifier genesisRequired() {
        if (!genesisInitialized) revert GenesisNotSet();
        _;
    }
    
    // ============ Constructor ============
    
    constructor(
        address _groth16Verifier,
        address _epochManager,
        address _rateLimiter,
        bytes32 _genesisLineageCommitment,
        bytes32 _policyRoot,
        bool _allowDuplicates
    ) {
        if (_groth16Verifier == address(0)) revert ZeroAddress();
        if (_epochManager == address(0)) revert ZeroAddress();
        if (_rateLimiter == address(0)) revert ZeroAddress();
        
        groth16Verifier = Groth16Verifier(_groth16Verifier);
        epochManager = EpochManager(_epochManager);
        rateLimiter = RateLimiter(_rateLimiter);
        
        admin = msg.sender;
        currentPolicyRoot = _policyRoot;
        genesisLineageCommitment = _genesisLineageCommitment;
        allowDuplicateStates = _allowDuplicates;
        paused = false;
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Initialize genesis state
     * @param _genesisStateHash Genesis state hash
     * @param _genesisLineageCommitment Genesis lineage commitment
     */
    function setGenesis(
        bytes32 _genesisStateHash,
        bytes32 _genesisLineageCommitment
    ) external override onlyAdmin {
        if (genesisInitialized) revert GenesisAlreadySet();
        if (_genesisStateHash == bytes32(0)) revert ZeroStateHash();
        
        genesisStateHash = _genesisStateHash;
        genesisLineageCommitment = _genesisLineageCommitment;
        
        stateLineage[_genesisStateHash] = _genesisLineageCommitment;
        stateDepth[_genesisStateHash] = 0;
        verifiedStates[_genesisStateHash] = true;
        stateOriginClass[_genesisStateHash] = ORIGIN_GENESIS;
        stateTimestamp[_genesisStateHash] = block.timestamp;
        stateCreator[_genesisStateHash] = msg.sender;
        
        genesisInitialized = true;
        
        emit GenesisSet(_genesisStateHash, _genesisLineageCommitment, msg.sender);
    }
    
    /**
     * @notice Update policy root
     * @param _newPolicyRoot New policy root
     */
    function updatePolicyRoot(bytes32 _newPolicyRoot) external onlyAdmin {
        currentPolicyRoot = _newPolicyRoot;
    }
    
    /**
     * @notice Transfer admin role (two-step process)
     * @param _newAdmin New admin address
     */
    function transferAdmin(address _newAdmin) external onlyAdmin {
        if (_newAdmin == address(0)) revert ZeroAddress();
        pendingAdmin = _newAdmin;
    }
    
    /**
     * @notice Accept admin role transfer
     */
    function acceptAdmin() external {
        if (msg.sender != pendingAdmin) revert NotPendingAdmin();
        admin = pendingAdmin;
        pendingAdmin = address(0);
    }
    
    /**
     * @notice Pause/unpause contract
     * @param _paused Whether to pause
     */
    function setPaused(bool _paused) external onlyAdmin {
        paused = _paused;
    }
    
    // ============ Core Verification ============
    
    /**
     * @notice Verify a lineage proof (12 public signals)
     * @param pA Groth16 proof point A
     * @param pB Groth16 proof point B
     * @param pC Groth16 proof point C
     * @param publicSignals 12 public signals from circuit
     * @return success Whether verification succeeded
     */
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[12] calldata publicSignals
    ) external override whenNotPaused genesisRequired returns (bool) {
        
        // Parse public signals
        PublicSignals memory signals = _parseSignals(publicSignals);
        
        // Generate proof hash for replay protection
        bytes32 proofHash = keccak256(abi.encodePacked(pA, pB, pC, publicSignals));
        if (usedProofs[proofHash]) revert ProofAlreadyUsed();
        
        // Mark proof as used
        usedProofs[proofHash] = true;
        
        // Validate all inputs
        _validateSignals(signals);
        
        // Verify Groth16 proof
        if (!groth16Verifier.verifyProof(pA, pB, pC, publicSignals)) {
            revert InvalidProof();
        }
        
        // Get previous state depth
        uint256 prevDepth = stateDepth[signals.prevStateHash];
        uint256 newDepth = prevDepth + 1;
        
        if (newDepth > MAX_DEPTH) revert MaxDepthExceeded();
        
        // Record new state
        _recordState(
            signals.newStateHash,
            signals.newLineageCommitment,
            newDepth,
            uint8(signals.newOriginClass)
        );
        
        totalTransitions++;
        if (newDepth > maxDepthReached) {
            maxDepthReached = newDepth;
        }
        
        emit LineageVerified(
            signals.prevStateHash,
            signals.newStateHash,
            signals.newLineageCommitment,
            newDepth,
            uint8(signals.newOriginClass),
            signals.epochId,
            msg.sender
        );
        
        return true;
    }
    
    // ============ Internal Functions ============
    
    /**
     * @notice Parse 12 public signals into struct
     */
    function _parseSignals(uint256[12] calldata signals)
        internal
        pure
        returns (PublicSignals memory)
    {
        return PublicSignals({
            prevLineageCommitment: bytes32(signals[0]),
            newLineageCommitment: bytes32(signals[1]),
            policyRoot: bytes32(signals[2]),
            prevStateHash: bytes32(signals[3]),
            newStateHash: bytes32(signals[4]),
            prevCounterCommitment: bytes32(signals[5]),
            newCounterCommitment: bytes32(signals[6]),
            epochId: signals[7],
            prevOriginClass: signals[8],
            newOriginClass: signals[9],
            timestamp: signals[10],
            lineageValid: signals[11]
        });
    }
    
    /**
     * @notice Validate all signal constraints
     */
    function _validateSignals(PublicSignals memory signals) internal view {
        // 1. Validate state hashes are non-zero
        if (signals.newStateHash == bytes32(0)) revert ZeroStateHash();
        if (signals.prevStateHash == bytes32(0)) revert ZeroStateHash();
        
        // 2. Check duplicate state prevention
        if (!allowDuplicateStates && verifiedStates[signals.newStateHash]) {
            revert StateAlreadyExists();
        }
        
        // 3. Verify previous state exists
        if (!verifiedStates[signals.prevStateHash]) {
            revert PreviousStateNotVerified();
        }
        
        // 4. Verify lineage commitment matches
        if (stateLineage[signals.prevStateHash] != signals.prevLineageCommitment) {
            revert LineageMismatch();
        }
        
        // 5. Verify policy root matches
        if (signals.policyRoot != currentPolicyRoot) {
            revert PolicyMismatch();
        }
        
        // 6. Validate origin classes
        if (signals.prevOriginClass > 6) revert InvalidOriginClass();
        if (signals.newOriginClass > 6) revert InvalidOriginClass();
        
        // 7. Verify lineageValid flag
        if (signals.lineageValid != 1) revert InvalidLineageValid();
    }
    
    /**
     * @notice Record verified state
     */
    function _recordState(
        bytes32 _stateHash,
        bytes32 _lineageCommitment,
        uint256 _depth,
        uint8 _originClass
    ) internal {
        stateLineage[_stateHash] = _lineageCommitment;
        stateDepth[_stateHash] = _depth;
        verifiedStates[_stateHash] = true;
        stateOriginClass[_stateHash] = _originClass;
        stateTimestamp[_stateHash] = block.timestamp;
        stateCreator[_stateHash] = msg.sender;
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Get lineage commitment for state
     */
    function getLineage(bytes32 stateHash)
        external
        view
        override
        returns (bytes32)
    {
        return stateLineage[stateHash];
    }
    
    /**
     * @notice Check if state has verified lineage
     */
    function hasVerifiedLineage(bytes32 stateHash)
        external
        view
        override
        returns (bool)
    {
        return verifiedStates[stateHash];
    }
    
    /**
     * @notice Get depth of state
     */
    function getDepth(bytes32 stateHash)
        external
        view
        override
        returns (uint256)
    {
        return stateDepth[stateHash];
    }
    
    /**
     * @notice Check if proof hash has been used
     */
    function isProofUsed(bytes32 proofHash) external view returns (bool) {
        return usedProofs[proofHash];
    }
    
    /**
     * @notice Get origin class of state
     */
    function getOriginClass(bytes32 stateHash) external view returns (uint8) {
        return stateOriginClass[stateHash];
    }
    
    /**
     * @notice Get state creator
     */
    function getStateCreator(bytes32 stateHash) external view returns (address) {
        return stateCreator[stateHash];
    }
    
    /**
     * @notice Get stats
     */
    function getStats() external view returns (
        uint256 transitions,
        uint256 maxDepth,
        bool isInitialized,
        bool isPaused
    ) {
        return (totalTransitions, maxDepthReached, genesisInitialized, paused);
    }
}