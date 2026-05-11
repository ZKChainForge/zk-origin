// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./interfaces/ILineageVerifier.sol";
import "./interfaces/IAuthorizationVerifier.sol";
import "./Groth16Verifier.sol";
import "./EpochManager.sol";
import "./RateLimiter.sol";

contract LineageVerifier is ILineageVerifier {
    
    uint256 public constant MAX_DEPTH = 1_000_000;
    uint256 public constant VERSION = 2;
    
    uint8 public constant ORIGIN_GENESIS = 0;
    uint8 public constant ORIGIN_USER = 1;
    uint8 public constant ORIGIN_ADMIN = 2;
    uint8 public constant ORIGIN_BRIDGE = 3;
    uint8 public constant ORIGIN_GOVERNANCE = 4;
    uint8 public constant ORIGIN_SYSTEM = 5;
    uint8 public constant ORIGIN_EMERGENCY = 6;
    
    Groth16Verifier public immutable groth16Verifier;
    EpochManager public immutable epochManager;
    RateLimiter public immutable rateLimiter;
    IAuthorizationVerifier public immutable authVerifier;
    
    address public admin;
    address public pendingAdmin;
    
    bool public genesisInitialized;
    bytes32 public genesisStateHash;
    bytes32 public genesisLineageCommitment;
    bytes32 public currentPolicyRoot;
    bool public isPaused;
    
    bool[7][7] public policyMatrix;
    
    mapping(bytes32 => bytes32) public stateLineage;
    mapping(bytes32 => uint256) public stateDepth;
    mapping(bytes32 => bool) public verifiedStates;
    mapping(bytes32 => uint8) public stateOriginClass;
    mapping(bytes32 => uint256) public stateTimestamp;
    mapping(bytes32 => address) public stateCreator;
    mapping(bytes32 => bool) public usedProofs;
    
    mapping(uint256 => bytes32) public epochCounterCommitments;
    
    uint256 public totalTransitions;
    uint256 public maxDepthReached;
    uint256 public lastEpochProcessed;
    
    event AuthorizationVerified(uint8 indexed originClass, address indexed creator, bytes32 authCommitment);
    event AdminTransferred(address indexed newAdmin);
    event ContractPausedChanged(bool isPausedNow);
    
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
    error GenesisHashMismatch();
    error InvalidLineageValid();
    
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
    
    function setGenesis(bytes32 _genesisStateHash, bytes32 _genesisLineageCommitment) 
        external 
        override 
        onlyAdmin 
    {
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
        
        uint256 currentEpoch = epochManager.getCurrentEpoch();
        rateLimiter.resetCountersForEpoch(currentEpoch);
        
        genesisInitialized = true;
        
        emit GenesisSet(_genesisStateHash, _genesisLineageCommitment, msg.sender);
    }
    
    function _extractSignals(uint256[20] memory signals)
        internal pure returns (
            bytes32 prevStateHash,
            bytes32 newStateHash,
            uint256 epochId,
            uint8 prevOriginClass,
            uint8 newOriginClass,
            bytes32 prevLineageCommitment,
            bytes32 prevCounterCommitment,
            bytes32 policyRoot,
            bytes32 expectedGenesisHash,
            bytes32 authMessageHash,
            uint256[7] memory counterValues,
            bytes32 newLineageCommitment,
            bytes32 newCounterCommitment,
            uint256 lineageValid
        )
    {
        prevStateHash = bytes32(signals[0]);
        newStateHash = bytes32(signals[1]);
        epochId = signals[2];
        prevOriginClass = uint8(signals[3]);
        newOriginClass = uint8(signals[4]);
        prevLineageCommitment = bytes32(signals[5]);
        prevCounterCommitment = bytes32(signals[6]);
        policyRoot = bytes32(signals[7]);
        expectedGenesisHash = bytes32(signals[8]);
        authMessageHash = bytes32(signals[9]);
        
        for (uint256 i = 0; i < 7; i++) {
            counterValues[i] = signals[10 + i];
        }
        
        newLineageCommitment = bytes32(signals[17]);
        newCounterCommitment = bytes32(signals[18]);
        lineageValid = signals[19];
        
        if (prevOriginClass >= 7) revert InvalidOriginClass();
        if (newOriginClass >= 7) revert InvalidOriginClass();
        if (lineageValid != 1) revert InvalidProof();
        
        for (uint256 i = 0; i < 7; i++) {
            if (counterValues[i] > type(uint32).max) {
                revert InvalidCounterSignals();
            }
        }
    }
    
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
        if (verifiedStates[newStateHash]) revert StateAlreadyExists();
        if (!verifiedStates[prevStateHash]) revert PreviousStateNotVerified();
        if (stateLineage[prevStateHash] != prevLineageCommitment) revert LineageMismatch();
        if (policyRootSignal != currentPolicyRoot) revert PolicyMismatch();
        
        uint8 actualPrevOriginClass = stateOriginClass[prevStateHash];
        if (actualPrevOriginClass != prevOriginClass) {
            revert OriginPolicyViolated(actualPrevOriginClass, prevOriginClass);
        }
        
        if (!policyMatrix[prevOriginClass][newOriginClass]) {
            revert OriginPolicyViolated(prevOriginClass, newOriginClass);
        }
    }
    
    function _handleEpochTransition(uint256 epochId) internal {
        if (epochId > lastEpochProcessed) {
            rateLimiter.resetCountersForEpoch(epochId);
            lastEpochProcessed = epochId;
            emit EpochTransition(epochId - 1, epochId, block.timestamp);
        }
    }
    
    function _verifyEpochAndRateLimits(
        uint256 epochId,
        bytes32 prevCounterCommit,
        bytes32 newCounterCommit
    ) internal {
        uint256 currentEpoch = epochManager.getCurrentEpoch();
        
        if (currentEpoch > 0 && epochId < currentEpoch - 1) {
            revert EpochMismatch(currentEpoch, epochId);
        }
        if (epochId > currentEpoch) {
            revert EpochMismatch(currentEpoch, epochId);
        }
        
        _handleEpochTransition(epochId);
        
        bytes32 storedCommit = epochCounterCommitments[epochId];
        if (storedCommit != bytes32(0) && storedCommit != prevCounterCommit) {
            revert CounterCommitmentMismatch();
        }
        
        epochCounterCommitments[epochId] = newCounterCommit;
    }
    
    function _verifyAuthorization(uint8 originClass, bytes calldata authData) 
        internal 
        returns (bytes32 authCommitment) 
    {
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
        
        bool valid = authVerifier.verifyAuthorization(authType, authData);
        if (!valid) {
            revert AuthorizationFailed("Auth verification failed");
        }
        
        authCommitment = authVerifier.getAuthorizationCommitment(authType, authData);
        emit AuthorizationVerified(originClass, msg.sender, authCommitment);
    }
    
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
    
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[20] calldata publicSignals,
        uint8 authType,
        bytes calldata authData
    ) external override whenNotPaused genesisRequired returns (bool) {
        
        uint256[20] memory signals;
        for (uint256 i = 0; i < 20; i++) {
            signals[i] = publicSignals[i];
        }
        
        (
            bytes32 prevStateHash,
            bytes32 newStateHash,
            uint256 epochId,
            uint8 prevOriginClass,
            uint8 newOriginClass,
            bytes32 prevLineageCommitment,
            bytes32 prevCounterCommitment,
            bytes32 policyRoot,
            bytes32 expectedGenesisHash,
            bytes32 authMessageHash,
            uint256[7] memory counterValues,
            bytes32 newLineageCommitment,
            bytes32 newCounterCommitment,
            uint256 lineageValid
        ) = _extractSignals(signals);
        
        if (expectedGenesisHash != genesisStateHash) {
            revert GenesisHashMismatch();
        }
        
        if (lineageValid != 1) {
            revert InvalidLineageValid();
        }
        
        if (authMessageHash == bytes32(0)) {
            revert InvalidProof();
        }
        
        for (uint256 i = 0; i < 7; i++) {
            if (counterValues[i] > type(uint32).max) {
                revert InvalidCounterSignals();
            }
        }
        
        bytes32 proofHash = keccak256(abi.encode(pA, pB, pC, publicSignals, authType, authData));
        if (usedProofs[proofHash]) revert ProofAlreadyUsed();
        usedProofs[proofHash] = true;
        
        _verifyPreconditions(
            prevStateHash,
            newStateHash,
            prevLineageCommitment,
            policyRoot,
            prevOriginClass,
            newOriginClass
        );
        
        {
            uint256[17] memory groth16Inputs;
            for (uint256 i = 0; i < 17; i++) {
                groth16Inputs[i] = publicSignals[i];
            }
            
            if (!_verifyGroth16(pA, pB, pC, groth16Inputs)) {
                emit ProofRejected(proofHash, "Groth16 verification failed");
                revert InvalidProof();
            }
        }
        
        bytes32 authCommitment = _verifyAuthorization(newOriginClass, authData);
        
        _verifyEpochAndRateLimits(epochId, prevCounterCommitment, newCounterCommitment);
        
        uint256 prevDepth = stateDepth[prevStateHash];
        _recordState(newStateHash, newLineageCommitment, newOriginClass, prevDepth);
        
        rateLimiter.incrementCounter(epochId, newOriginClass);
        rateLimiter.storeCounterCommitment(epochId, newCounterCommitment);
        
        emit LineageVerified(
            prevStateHash,
            newStateHash,
            newLineageCommitment,
            prevDepth + 1,
            newOriginClass,
            epochId,
            msg.sender,
            authCommitment
        );
        
        return true;
    }
    
    function _verifyGroth16(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[17] memory publicInputs
    ) internal view returns (bool) {
        uint256[12] memory inputs12;
        for (uint256 i = 0; i < 12; i++) {
            inputs12[i] = publicInputs[i];
        }
        return groth16Verifier.verifyProof(pA, pB, pC, inputs12);
    }
    
    function getLineage(bytes32 stateHash) external view override returns (bytes32) {
        return stateLineage[stateHash];
    }
    
    function hasVerifiedLineage(bytes32 stateHash) external view override returns (bool) {
        return verifiedStates[stateHash];
    }
    
    function getDepth(bytes32 stateHash) external view override returns (uint256) {
        return stateDepth[stateHash];
    }
    
    function updatePolicyRoot(bytes32 _newPolicyRoot) external onlyAdmin {
        currentPolicyRoot = _newPolicyRoot;
    }
    
    function setPolicyTransition(uint8 from, uint8 to, bool allowed) external onlyAdmin {
        if (from >= 7 || to >= 7) revert InvalidOriginClass();
        policyMatrix[from][to] = allowed;
        emit PolicyUpdated(from, to, allowed);
    }
    
    function transferAdmin(address _newAdmin) external onlyAdmin {
        if (_newAdmin == address(0)) revert ZeroAddress();
        pendingAdmin = _newAdmin;
    }
    
    function acceptAdmin() external {
        if (msg.sender != pendingAdmin) revert NotPendingAdmin();
        admin = pendingAdmin;
        pendingAdmin = address(0);
        emit AdminTransferred(admin);
    }
    
    function setPaused(bool _paused) external onlyAdmin {
        isPaused = _paused;
        emit ContractPausedChanged(_paused);
    }
}