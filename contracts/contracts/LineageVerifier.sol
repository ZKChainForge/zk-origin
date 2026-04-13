// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/ILineageVerifier.sol";
import "./Groth16Verifier.sol";
import "./EpochManager.sol";
import "./RateLimiter.sol";
import "./AuthorizationVerifier.sol";

/**
 * @title LineageVerifier
 * @notice Core contract for ZK-ORIGIN state lineage verification
 * @dev Verifies state transitions using Groth16 zero-knowledge proofs
 * 
 * PRODUCTION VERSION - All security fixes applied
 */
contract LineageVerifier is ILineageVerifier {
    
    // ============ Constants ============
    
    uint256 public constant MAX_DEPTH = 1_000_000;
    uint256 public constant VERSION = 1;
    
    // Origin classes
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
    AuthorizationVerifier public immutable authVerifier;
    
    // ============ Mutable State ============
    
    address public admin;
    address public pendingAdmin;
    
    bool public genesisInitialized;
    bytes32 public genesisStateHash;
    bytes32 public genesisLineageCommitment;
    
    bytes32 public currentPolicyRoot;
    bool public paused;
    bool public allowDuplicateStates;
    
    // Policy enforcement matrix
    bool[7][7] public policyMatrix;
    
    // State tracking
    mapping(bytes32 => bytes32) public stateLineage;
    mapping(bytes32 => uint256) public stateDepth;
    mapping(bytes32 => bool) public verifiedStates;
    mapping(bytes32 => uint8) public stateOriginClass;
    mapping(bytes32 => uint256) public stateTimestamp;
    mapping(bytes32 => address) public stateCreator;
    mapping(bytes32 => bool) public usedProofs;
    
    // ADDED: Counter commitment tracking
    mapping(uint256 => bytes32) public epochCounterCommitments;
    
    // Statistics
    uint256 public totalTransitions;
    uint256 public maxDepthReached;
    
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
    error EpochMismatchError(uint256 expected, uint256 actual);
    error RateLimitExceededError(uint8 originClass);
    error OriginPolicyViolatedError(uint8 from, uint8 to);
    error AuthorizationFailedError();
    error LineageValidFlagError();
    error CounterCommitmentMismatch();
    
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
        address _authVerifier,
        bytes32 _genesisLineageCommitment,
        bytes32 _policyRoot,
        bool _allowDuplicates
    ) {
        if (_groth16Verifier == address(0)) revert ZeroAddress();
        if (_epochManager == address(0)) revert ZeroAddress();
        if (_rateLimiter == address(0)) revert ZeroAddress();
        if (_authVerifier == address(0)) revert ZeroAddress();
        
        groth16Verifier = Groth16Verifier(_groth16Verifier);
        epochManager = EpochManager(_epochManager);
        rateLimiter = RateLimiter(_rateLimiter);
        authVerifier = AuthorizationVerifier(_authVerifier);
        
        admin = msg.sender;
        currentPolicyRoot = _policyRoot;
        genesisLineageCommitment = _genesisLineageCommitment;
        allowDuplicateStates = _allowDuplicates;
        paused = false;
        
        _initializeDefaultPolicy();
    }
    
    // ============ Policy Initialization ============
    
    function _initializeDefaultPolicy() internal {
        // Genesis → User, Admin, System
        policyMatrix[ORIGIN_GENESIS][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_GENESIS][ORIGIN_ADMIN] = true;
        policyMatrix[ORIGIN_GENESIS][ORIGIN_SYSTEM] = true;
        
        // User → User
        policyMatrix[ORIGIN_USER][ORIGIN_USER] = true;
        
        // Admin → User, Admin, Bridge, System
        policyMatrix[ORIGIN_ADMIN][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_ADMIN] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_BRIDGE] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_SYSTEM] = true;
        
        // Bridge → User
        policyMatrix[ORIGIN_BRIDGE][ORIGIN_USER] = true;
        
        // Governance → ALL
        for (uint8 i = 0; i < 7; i++) {
            policyMatrix[ORIGIN_GOVERNANCE][i] = true;
        }
        
        // System → User, System
        policyMatrix[ORIGIN_SYSTEM][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_SYSTEM][ORIGIN_SYSTEM] = true;
        
        // Emergency → User, Admin, System
        policyMatrix[ORIGIN_EMERGENCY][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_EMERGENCY][ORIGIN_ADMIN] = true;
        policyMatrix[ORIGIN_EMERGENCY][ORIGIN_SYSTEM] = true;
    }
    
    // ============ Admin Functions ============
    
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
    
    function updatePolicyRoot(bytes32 _newPolicyRoot) external onlyAdmin {
        currentPolicyRoot = _newPolicyRoot;
    }
    
    function setPolicyTransition(
        uint8 from,
        uint8 to,
        bool allowed
    ) external onlyAdmin {
        if (from >= 7 || to >= 7) revert InvalidOriginClass();
        policyMatrix[from][to] = allowed;
    }
    
    function transferAdmin(address _newAdmin) external onlyAdmin {
        if (_newAdmin == address(0)) revert ZeroAddress();
        pendingAdmin = _newAdmin;
    }
    
    function acceptAdmin() external {
        if (msg.sender != pendingAdmin) revert NotPendingAdmin();
        admin = pendingAdmin;
        pendingAdmin = address(0);
    }
    
    function setPaused(bool _paused) external onlyAdmin {
        paused = _paused;
    }
    
    // ============ Core Verification (FIXED) ============
    
/**
 * @notice Extract and validate public signals
 * 
 * CORRECT Signal Order (OUTPUTS FIRST!):
 * [0]  newLineageCommitment     (output)
 * [1]  newCounterCommitment     (output)
 * [2]  lineageValid             (output)
 * [3]  prevStateHash            (input)
 * [4]  newStateHash             (input)
 * [5]  epochId                  (input)
 * [6]  prevOriginClass          (input)
 * [7]  newOriginClass           (input)
 * [8]  prevLineageCommitment    (input)
 * [9]  prevCounterCommitment    (input)
 * [10] policyRoot               (input)
 * [11] expectedGenesisHash      (input)
 */
function _extractSignals(uint256[12] calldata signals)
    internal
    pure
    returns (
        bytes32 prevLineage,
        bytes32 newLineage,
        bytes32 policyRoot,
        bytes32 prevStateHash,
        bytes32 newStateHash,
        bytes32 prevCounterCommit,
        bytes32 newCounterCommit,
        uint8 prevOriginClass,
        uint8 newOriginClass,
        uint256 epochId,
        uint256 lineageValid
    )
{
    // OUTPUTS FIRST
    newLineage = bytes32(signals[0]);
    newCounterCommit = bytes32(signals[1]);
    lineageValid = signals[2];
    
    // THEN INPUTS
    prevStateHash = bytes32(signals[3]);
    newStateHash = bytes32(signals[4]);
    epochId = signals[5];
    prevOriginClass = uint8(signals[6]);
    newOriginClass = uint8(signals[7]);
    prevLineage = bytes32(signals[8]);
    prevCounterCommit = bytes32(signals[9]);
    policyRoot = bytes32(signals[10]);
    // signals[11] = expectedGenesisHash (not used by contract)
    
    if (prevOriginClass > 6) revert InvalidOriginClass();
    if (newOriginClass > 6) revert InvalidOriginClass();
    if (lineageValid != 1) revert LineageValidFlagError();
}
    
    /**
     * @notice Helper: Verify all preconditions
     */
    function _verifyPreconditions(
        bytes32 prevStateHash,
        bytes32 newStateHash,
        bytes32 prevLineageCommitment,
        bytes32 policyRootSignal,
        uint8 prevOriginClass,
        uint8 newOriginClass
    ) internal view {
        if (newStateHash == bytes32(0)) revert ZeroStateHash();
        if (prevStateHash == bytes32(0)) revert ZeroStateHash();
        
        if (!allowDuplicateStates && verifiedStates[newStateHash]) {
            revert StateAlreadyExists();
        }
        
        if (!verifiedStates[prevStateHash]) {
            revert PreviousStateNotVerified();
        }
        
        if (stateLineage[prevStateHash] != prevLineageCommitment) {
            revert LineageMismatch();
        }
        
        if (policyRootSignal != currentPolicyRoot) {
            revert PolicyMismatch();
        }
        
        uint8 actualPrevOriginClass = stateOriginClass[prevStateHash];
        if (actualPrevOriginClass != prevOriginClass) {
            revert OriginPolicyViolatedError(actualPrevOriginClass, prevOriginClass);
        }
        
        if (!policyMatrix[prevOriginClass][newOriginClass]) {
            revert OriginPolicyViolatedError(prevOriginClass, newOriginClass);
        }
    }
    
    /**
     * @notice Helper: Verify and update epoch/rate limits
     * FIXED: Allow proofs from previous epoch (1 epoch grace period)
     */
    function _verifyAndUpdateEpoch(uint8 newOriginClass, uint256 epochId) internal {
        uint256 currentEpoch = epochManager.getCurrentEpoch();
        
        // FIXED: Allow proofs from current epoch or 1 epoch ago
        if (epochId < currentEpoch - 1 || epochId > currentEpoch) {
            revert EpochMismatchError(currentEpoch, epochId);
        }
        
        epochManager.updateEpoch();
        
        if (!rateLimiter.canProceed(epochId, newOriginClass)) {
            revert RateLimitExceededError(newOriginClass);
        }
    }
    
    /**
     * @notice Helper: Record new state
     */
    function _recordState(
        bytes32 newStateHash,
        bytes32 newLineageCommitment,
        uint8 newOriginClass,
        uint256 prevDepth
    ) internal {
        uint256 newDepth = prevDepth + 1;
        if (newDepth > MAX_DEPTH) revert MaxDepthExceeded();
        
        stateLineage[newStateHash] = newLineageCommitment;
        stateDepth[newStateHash] = newDepth;
        verifiedStates[newStateHash] = true;
        stateOriginClass[newStateHash] = newOriginClass;
        stateTimestamp[newStateHash] = block.timestamp;
        stateCreator[newStateHash] = msg.sender;
        
        totalTransitions++;
        if (newDepth > maxDepthReached) {
            maxDepthReached = newDepth;
        }
    }
    
    /**
     * @notice Main verification function (PRODUCTION VERSION)
     * 
     * FIXES APPLIED:
     * 1. Proof hash includes publicSignals (prevents replay with different signals)
     * 2. Counter commitment verification (ensures circuit computed correct counters)
     * 3. Epoch grace period (allows proofs from previous epoch)
     */
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[12] calldata publicSignals
    ) external override whenNotPaused genesisRequired returns (bool) {
        
        // Extract and validate signals
        (
            bytes32 prevLineage,
            bytes32 newLineage,
            bytes32 policyRoot,
            bytes32 prevStateHash,
            bytes32 newStateHash,
            bytes32 prevCounterCommit,
            bytes32 newCounterCommit,
            uint8 prevOriginClass,
            uint8 newOriginClass,
            uint256 epochId,
        ) = _extractSignals(publicSignals);
        
        // FIXED: Verify counter commitment consistency
        bytes32 storedCounterCommit = epochCounterCommitments[epochId];
        if (storedCounterCommit != bytes32(0) && storedCounterCommit != prevCounterCommit) {
            revert CounterCommitmentMismatch();
        }
        
        // FIXED: Compute proof hash including public signals
        bytes32 proofHash = keccak256(abi.encode(pA, pB, pC, publicSignals));
        if (usedProofs[proofHash]) revert ProofAlreadyUsed();
        usedProofs[proofHash] = true;
        
        // Verify all preconditions
        _verifyPreconditions(
            prevStateHash,
            newStateHash,
            prevLineage,
            policyRoot,
            prevOriginClass,
            newOriginClass
        );
        
        // Verify Groth16 proof
        if (!groth16Verifier.verifyProof(pA, pB, pC, publicSignals)) {
            revert InvalidProof();
        }
        
        // FIXED: Verify and update epoch/rate limits (with grace period)
        _verifyAndUpdateEpoch(newOriginClass, epochId);
        
        // Get previous depth
        uint256 prevDepth = stateDepth[prevStateHash];
        
        // Record new state
        _recordState(newStateHash, newLineage, newOriginClass, prevDepth);
        
        // Increment rate limiter
        rateLimiter.incrementCounter(epochId, newOriginClass);
        
        // FIXED: Store new counter commitment
        epochCounterCommitments[epochId] = newCounterCommit;
        
        // Emit event
        emit LineageVerified(
            prevStateHash,
            newStateHash,
            newLineage,
            prevDepth + 1,
            newOriginClass,
            epochId,
            msg.sender
        );
        
        return true;
    }
    
    // ============ View Functions ============
    
    function getLineage(bytes32 stateHash)
        external
        view
        override
        returns (bytes32)
    {
        return stateLineage[stateHash];
    }
    
    function hasVerifiedLineage(bytes32 stateHash)
        external
        view
        override
        returns (bool)
    {
        return verifiedStates[stateHash];
    }
    
    function getDepth(bytes32 stateHash)
        external
        view
        override
        returns (uint256)
    {
        return stateDepth[stateHash];
    }
    
    function isTransitionAllowed(uint8 from, uint8 to)
        external
        view
        returns (bool)
    {
        if (from >= 7 || to >= 7) return false;
        return policyMatrix[from][to];
    }
    
    function getOriginClass(bytes32 stateHash)
        external
        view
        returns (uint8)
    {
        return stateOriginClass[stateHash];
    }
    
    function getStateCreator(bytes32 stateHash)
        external
        view
        returns (address)
    {
        return stateCreator[stateHash];
    }
    
    function isProofUsed(bytes32 proofHash)
        external
        view
        returns (bool)
    {
        return usedProofs[proofHash];
    }
    
    /**
     * @notice Get counter commitment for an epoch
     */
    function getCounterCommitment(uint256 epochId)
        external
        view
        returns (bytes32)
    {
        return epochCounterCommitments[epochId];
    }
    
    function getStats() external view returns (
        uint256 transitions,
        uint256 maxDepth,
        bool initialized,
        bool isPaused,
        uint256 currentEpoch
    ) {
        return (
            totalTransitions,
            maxDepthReached,
            genesisInitialized,
            paused,
            epochManager.getCurrentEpoch()
        );
    }
}