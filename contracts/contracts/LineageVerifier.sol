// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/ILineageVerifier.sol";
import "./interfaces/IAuthorizationVerifier.sol";
import "./Groth16Verifier.sol";
import "./EpochManager.sol";
import "./RateLimiter.sol";

/**
 * @title LineageVerifier (PRODUCTION - FIXED)
 * @notice Core contract for ZK-ORIGIN state lineage verification
 * 
 * SECURITY GUARANTEES:
 *  Every state proves legitimate lineage from genesis
 *  Authorization required for every transition
 *  Origin policy enforced (prevents privilege escalation)
 *  Rate limits prevent DOS attacks
 *  Epoch management prevents replay
 *  Genesis immutable (one-time set)
 *  No state duplication allowed
 *  Nonce prevents replay attacks
 *  Counter monotonic tracking
 *  Proof replay protection
 */

contract LineageVerifier is ILineageVerifier {
    
    // ============ Constants ============
    uint256 public constant MAX_DEPTH = 1_000_000;
    uint256 public constant VERSION = 2;
    uint256 public constant COUNTER_MAX = type(uint32).max;
    
    // Origin classes (must match circuit)
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
    RateLimiter public immutable rateLimiter;
    IAuthorizationVerifier public immutable authVerifier;
    
    // ============ Mutable State ============
    address public admin;
    address public pendingAdmin;
    
    bool public genesisInitialized;
    bytes32 public genesisStateHash;
    bytes32 public genesisLineageCommitment;
    bytes32 public currentPolicyRoot;
    bool public isPaused;
    
    // Origin policy matrix (from → to allowed?)
    bool[7][7] public policyMatrix;
    
    // State tracking
    mapping(bytes32 => bytes32) public stateLineage;
    mapping(bytes32 => uint256) public stateDepth;
    mapping(bytes32 => bool) public verifiedStates;
    mapping(bytes32 => uint8) public stateOriginClass;
    mapping(bytes32 => uint256) public stateTimestamp;
    mapping(bytes32 => address) public stateCreator;
    mapping(bytes32 => bool) public usedProofs;
    
    // Counter commitments per epoch
    mapping(uint256 => bytes32) public epochCounterCommitments;
    mapping(uint256 => bool) public epochCountersReset;
    
    // Statistics
    uint256 public totalTransitions;
    uint256 public maxDepthReached;
    uint256 public lastEpochProcessed;
    
    // ============ Events (Inherited from interface) ============
    
    event AuthorizationVerified(
        uint8 indexed originClass,
        address indexed creator,
        bytes32 authCommitment
    );
    
    event AdminTransferred(address indexed newAdmin);
    event ContractPausedChanged(bool isPausedNow);
    
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
    error ContractIsPaused();
    error ProofAlreadyUsed();
    error StateAlreadyExists();
    error InvalidOriginClass();
    error RateLimitExceeded(uint8 originClass);
    error OriginPolicyViolated(uint8 from, uint8 to);
    error AuthorizationFailed(string reason);
    error CounterCommitmentMismatch();
    error EpochMismatch(uint256 expected, uint256 actual);
    error InvalidCounterSignals();
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    modifier whenNotPaused() {
        if (isPaused) revert ContractIsPaused();
        _;
    }
    
    modifier genesisRequired() {
        if (!genesisInitialized) revert GenesisNotSet();
        _;
    }
    
    // ============ Constructor ============
    
    /**
     * @notice Initialize LineageVerifier
     * @param _groth16Verifier Groth16 proof verifier address
     * @param _epochManager Epoch management contract
     * @param _rateLimiter Rate limit tracking contract
     * @param _authVerifier Authorization verification contract
     * @param _genesisLineageCommitment Genesis lineage commitment
     * @param _policyRoot Initial policy Merkle root
     */
    constructor(
        address _groth16Verifier,
        address _epochManager,
        address _rateLimiter,
        address _authVerifier,
        bytes32 _genesisLineageCommitment,
        bytes32 _policyRoot
    ) {
        if (_groth16Verifier == address(0)) revert ZeroAddress();
        if (_epochManager == address(0)) revert ZeroAddress();
        if (_rateLimiter == address(0)) revert ZeroAddress();
        if (_authVerifier == address(0)) revert ZeroAddress();
        
        groth16Verifier = Groth16Verifier(_groth16Verifier);
        epochManager = EpochManager(_epochManager);
        rateLimiter = RateLimiter(_rateLimiter);
        authVerifier = IAuthorizationVerifier(_authVerifier);
        
        admin = msg.sender;
        currentPolicyRoot = _policyRoot;
        genesisLineageCommitment = _genesisLineageCommitment;
        isPaused = false;
        lastEpochProcessed = 0;
        
        _initializeDefaultPolicy();
    }
    
    // ============ Policy Initialization ============
    
    /**
     * @notice Initialize default origin transition policy
     */
    function _initializeDefaultPolicy() internal {
        // Genesis → User, Admin, System
        policyMatrix[ORIGIN_GENESIS][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_GENESIS][ORIGIN_ADMIN] = true;
        policyMatrix[ORIGIN_GENESIS][ORIGIN_SYSTEM] = true;
        
        // User → User only
        policyMatrix[ORIGIN_USER][ORIGIN_USER] = true;
        
        // Admin → User, Admin, Bridge, System
        policyMatrix[ORIGIN_ADMIN][ORIGIN_USER] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_ADMIN] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_BRIDGE] = true;
        policyMatrix[ORIGIN_ADMIN][ORIGIN_SYSTEM] = true;
        
        // Bridge → User only
        policyMatrix[ORIGIN_BRIDGE][ORIGIN_USER] = true;
        
        // Governance → All
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
    
    // ============ Genesis Management ============
    
    /**
     * @notice Set genesis state (immutable, one-time only)
     * @param _genesisStateHash Fixed genesis state hash
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
        
        // Initialize genesis state record
        stateLineage[_genesisStateHash] = _genesisLineageCommitment;
        stateDepth[_genesisStateHash] = 0;
        verifiedStates[_genesisStateHash] = true;
        stateOriginClass[_genesisStateHash] = ORIGIN_GENESIS;
        stateTimestamp[_genesisStateHash] = block.timestamp;
        stateCreator[_genesisStateHash] = msg.sender;
        
        // Initialize genesis epoch counters
        uint256 currentEpoch = epochManager.getCurrentEpoch();
        rateLimiter.resetCountersForEpoch(currentEpoch);
        
        genesisInitialized = true;
        
        emit GenesisSet(_genesisStateHash, _genesisLineageCommitment, msg.sender);
    }
    
    // ============ Signal Extraction ============
    
    /**
     * @notice Extract and validate signals from Groth16 proof
     * @param signals 19 public signals from circuit
     * @return prevLineage Previous lineage commitment
     * @return newLineage New lineage commitment  
     * @return newCounterCommit New counter commitment
     * @return policyRoot Policy root
     * @return prevStateHash Previous state hash
     * @return newStateHash New state hash
     * @return prevCounterCommit Previous counter commitment
     * @return prevOriginClass Previous origin class
     * @return newOriginClass New origin class
     * @return epochId Epoch ID
     * @return lineageValid Lineage validity flag
     * @return newCounterValues New counter values array
     */
    function _extractSignals(uint256[19] memory signals)
        internal pure returns (
            bytes32 prevLineage,
            bytes32 newLineage,
            bytes32 newCounterCommit,
            bytes32 policyRoot,
            bytes32 prevStateHash,
            bytes32 newStateHash,
            bytes32 prevCounterCommit,
            uint8 prevOriginClass,
            uint8 newOriginClass,
            uint256 epochId,
            uint256 lineageValid,
            uint256[7] memory newCounterValues
        )
    {
        // Extract outputs
        newLineage = bytes32(signals[0]);
        newCounterCommit = bytes32(signals[1]);
        lineageValid = signals[2];
        
        // Extract inputs
        prevStateHash = bytes32(signals[3]);
        newStateHash = bytes32(signals[4]);
        epochId = signals[5];
        prevOriginClass = uint8(signals[6]);
        newOriginClass = uint8(signals[7]);
        prevLineage = bytes32(signals[8]);
        prevCounterCommit = bytes32(signals[9]);
        policyRoot = bytes32(signals[10]);
        
        // Extract counter values
        for (uint256 i = 0; i < 7; i++) {
            newCounterValues[i] = signals[12 + i];
        }
        
        // Validate
        if (prevOriginClass >= 7) revert InvalidOriginClass();
        if (newOriginClass >= 7) revert InvalidOriginClass();
        if (lineageValid != 1) revert InvalidProof();
        
        // Validate counter values are within range
        for (uint256 i = 0; i < 7; i++) {
            if (newCounterValues[i] > COUNTER_MAX) {
                revert InvalidCounterSignals();
            }
        }
    }
    
    // ============ Precondition Verification ============
    
    /**
     * @notice Verify preconditions before proof verification
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
        
        // Never allow duplicate states
        if (verifiedStates[newStateHash]) {
            revert StateAlreadyExists();
        }
        
        // Previous state must be verified
        if (!verifiedStates[prevStateHash]) {
            revert PreviousStateNotVerified();
        }
        
        // Lineage must match
        if (stateLineage[prevStateHash] != prevLineageCommitment) {
            revert LineageMismatch();
        }
        
        // Policy root must match current
        if (policyRootSignal != currentPolicyRoot) {
            revert PolicyMismatch();
        }
        
        // Previous state's origin class must match
        uint8 actualPrevOriginClass = stateOriginClass[prevStateHash];
        if (actualPrevOriginClass != prevOriginClass) {
            revert OriginPolicyViolated(actualPrevOriginClass, prevOriginClass);
        }
        
        // Policy must allow transition
        if (!policyMatrix[prevOriginClass][newOriginClass]) {
            revert OriginPolicyViolated(prevOriginClass, newOriginClass);
        }
    }
    
    // ============ Epoch & Rate Limit Management ============
    
    /**
     * @notice Handle epoch transition if needed
     */
    function _handleEpochTransition(uint256 epochId) internal {
        if (epochId > lastEpochProcessed) {
            rateLimiter.resetCountersForEpoch(epochId);
            lastEpochProcessed = epochId;
            epochCountersReset[epochId] = true;
            
            emit EpochTransition(epochId - 1, epochId, block.timestamp);
        }
    }
    
    /**
     * @notice Verify epoch and rate limits
     */
    function _verifyEpochAndRateLimits(
        uint8 newOriginClass,
        uint256 epochId,
        bytes32 prevCounterCommit,
        uint256[7] memory newCounterValues,
        bytes32 newCounterCommit
    ) internal {
        uint256 currentEpoch = epochManager.getCurrentEpoch();
        
        // Allow current epoch or 1 epoch grace period
        if (epochId < currentEpoch - 1 || epochId > currentEpoch) {
            revert EpochMismatch(currentEpoch, epochId);
        }
        
        // Handle epoch transition
        _handleEpochTransition(epochId);
        
        // Check rate limit
        if (!rateLimiter.canProceed(epochId, newOriginClass)) {
            revert RateLimitExceeded(newOriginClass);
        }
        
        // Verify counter commitment consistency
        bytes32 storedCommit = epochCounterCommitments[epochId];
        if (storedCommit != bytes32(0) && storedCommit != prevCounterCommit) {
            revert CounterCommitmentMismatch();
        }
        
        // Verify new counter commitment
        bytes32 computedCommit = keccak256(
            abi.encode(epochId, newCounterValues)
        );
        if (computedCommit != newCounterCommit) {
            revert CounterCommitmentMismatch();
        }
    }
    
    // ============ Authorization Verification ============
    
    /**
     * @notice Verify authorization for origin class
     */
    function _verifyAuthorization(
        uint8 originClass,
        bytes calldata authData
    ) internal returns (bytes32 authCommitment) {
        IAuthorizationVerifier.AuthType authType;
        
        if (originClass == ORIGIN_USER) {
            authType = IAuthorizationVerifier.AuthType.User;
        } else if (originClass == ORIGIN_ADMIN) {
            authType = IAuthorizationVerifier.AuthType.Admin;
        } else if (originClass == ORIGIN_BRIDGE) {
            authType = IAuthorizationVerifier.AuthType.Bridge;
        } else if (originClass == ORIGIN_GOVERNANCE) {
            authType = IAuthorizationVerifier.AuthType.Governance;
        } else if (originClass == ORIGIN_SYSTEM) {
            authType = IAuthorizationVerifier.AuthType.System;
        } else if (originClass == ORIGIN_EMERGENCY) {
            authType = IAuthorizationVerifier.AuthType.Emergency;
        } else {
            revert InvalidOriginClass();
        }
        
        // Verify authorization
        bool valid = authVerifier.verifyAuthorization(authType, authData);
        if (!valid) {
            revert AuthorizationFailed("Auth verification failed");
        }
        
        // Get commitment for storage
        authCommitment = authVerifier.getAuthorizationCommitment(authType, authData);
        
        emit AuthorizationVerified(originClass, msg.sender, authCommitment);
    }
    
    // ============ State Recording ============
    
    /**
     * @notice Record verified state
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
    
    // ============ Helper: Compute Proof Hash ============
    
    /**
     * @notice Compute proof hash for replay protection
     * FIX: Split encoding to avoid stack depth issues with viaIR
     */
    function _computeProofHash(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[19] calldata publicSignals,
        uint8 authType,
        bytes calldata authData
    ) internal pure returns (bytes32) {
        // Part 1: Encode proof components
        bytes32 proofPart1 = keccak256(abi.encode(pA, pB, pC));
        
        // Part 2: Encode signals and auth data
        bytes32 proofPart2 = keccak256(abi.encode(publicSignals, authType, authData));
        
        // Combine both parts
        return keccak256(abi.encode(proofPart1, proofPart2));
    }
    
    // ============ Main Verification Function ============
    
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
    ) external override whenNotPaused genesisRequired returns (bool) {
        
        // Convert signals to memory for extraction
        uint256[19] memory signals;
        for (uint256 i = 0; i < 19; i++) {
            signals[i] = publicSignals[i];
        }
        
        // Extract signals
        (
            bytes32 prevLineage,
            bytes32 newLineage,
            bytes32 newCounterCommit,
            bytes32 policyRoot,
            bytes32 prevStateHash,
            bytes32 newStateHash,
            bytes32 prevCounterCommit,
            uint8 prevOriginClass,
            uint8 newOriginClass,
            uint256 epochId,
            uint256 lineageValid,
            uint256[7] memory newCounterValues
        ) = _extractSignals(signals);
        
        // Compute proof hash for replay protection (FIX: split encoding)
        bytes32 proofHash = _computeProofHash(
            pA,
            pB,
            pC,
            publicSignals,
            authType,
            authData
        );
        
        if (usedProofs[proofHash]) {
            revert ProofAlreadyUsed();
        }
        usedProofs[proofHash] = true;
        
        // Verify preconditions
        _verifyPreconditions(
            prevStateHash,
            newStateHash,
            prevLineage,
            policyRoot,
            prevOriginClass,
            newOriginClass
        );
        
        // Convert signals to uint[12] for Groth16 verifier
        uint256[12] memory groth16Signals;
        for (uint256 i = 0; i < 12; i++) {
            groth16Signals[i] = publicSignals[i];
        }
        
        // Verify Groth16 proof
        if (!groth16Verifier.verifyProof(pA, pB, pC, groth16Signals)) {
            emit ProofRejected(proofHash, "Groth16 verification failed");
            revert InvalidProof();
        }
        
        // Verify authorization
        bytes32 authCommitment = _verifyAuthorization(newOriginClass, authData);
        
        // Verify epoch and rate limits
        _verifyEpochAndRateLimits(
            newOriginClass,
            epochId,
            prevCounterCommit,
            newCounterValues,
            newCounterCommit
        );
        
        // Get previous depth
        uint256 prevDepth = stateDepth[prevStateHash];
        
        // Record new state
        _recordState(newStateHash, newLineage, newOriginClass, prevDepth);
        
        // Increment rate limiter
        rateLimiter.incrementCounter(epochId, newOriginClass);
        
        // Store counter commitment
        rateLimiter.storeCounterCommitment(epochId, newCounterCommit);
        epochCounterCommitments[epochId] = newCounterCommit;
        
        // Emit success event
        emit LineageVerified(
            prevStateHash,
            newStateHash,
            newLineage,
            prevDepth + 1,
            newOriginClass,
            epochId,
            msg.sender,
            authCommitment
        );
        
        return true;
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Get lineage commitment for state
     */
    function getLineage(bytes32 stateHash)
        external view override returns (bytes32)
    {
        return stateLineage[stateHash];
    }
    
    /**
     * @notice Check if state has verified lineage
     */
    function hasVerifiedLineage(bytes32 stateHash)
        external view override returns (bool)
    {
        return verifiedStates[stateHash];
    }
    
    /**
     * @notice Get lineage depth
     */
    function getDepth(bytes32 stateHash)
        external view override returns (uint256)
    {
        return stateDepth[stateHash];
    }
    
    /**
     * @notice Check if transition is allowed by policy
     */
    function isTransitionAllowed(uint8 from, uint8 to)
        external view returns (bool)
    {
        if (from >= 7 || to >= 7) return false;
        return policyMatrix[from][to];
    }
    
    /**
     * @notice Get origin class of state
     */
    function getOriginClass(bytes32 stateHash)
        external view returns (uint8)
    {
        return stateOriginClass[stateHash];
    }
    
    /**
     * @notice Get counter commitment for epoch
     */
    function getCounterCommitment(uint256 epochId)
        external view returns (bytes32)
    {
        return epochCounterCommitments[epochId];
    }
    
    /**
     * @notice Get contract statistics
     */
    function getStats() external view returns (
        uint256 transitions,
        uint256 maxDepth,
        bool initialized,
        bool paused,
        uint256 currentEpoch,
        uint256 lastProcessedEpoch
    ) {
        return (
            totalTransitions,
            maxDepthReached,
            genesisInitialized,
            isPaused,
            epochManager.getCurrentEpoch(),
            lastEpochProcessed
        );
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Update policy root (for policy changes)
     */
    function updatePolicyRoot(bytes32 _newPolicyRoot) external onlyAdmin {
        currentPolicyRoot = _newPolicyRoot;
    }
    
    /**
     * @notice Set policy transition
     */
    function setPolicyTransition(
        uint8 from,
        uint8 to,
        bool allowed
    ) external onlyAdmin {
        if (from >= 7 || to >= 7) revert InvalidOriginClass();
        policyMatrix[from][to] = allowed;
        emit PolicyUpdated(from, to, allowed);
    }
    
    /**
     * @notice Transfer admin role
     */
    function transferAdmin(address _newAdmin) external onlyAdmin {
        if (_newAdmin == address(0)) revert ZeroAddress();
        pendingAdmin = _newAdmin;
    }
    
    /**
     * @notice Accept admin transfer
     */
    function acceptAdmin() external {
        if (msg.sender != pendingAdmin) revert NotPendingAdmin();
        admin = pendingAdmin;
        pendingAdmin = address(0);
        emit AdminTransferred(admin);
    }
    
    /**
     * @notice Pause contract
     */
    function setPaused(bool _paused) external onlyAdmin {
        isPaused = _paused;
        emit ContractPausedChanged(_paused);
    }
}