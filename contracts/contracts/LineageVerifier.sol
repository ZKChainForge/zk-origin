// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/ILineageVerifier.sol";

interface IGroth16Verifier {
    function verifyProof(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[5] calldata pubSignals
    ) external view returns (bool);
}

interface IPolicyRegistry {
    function isValidPolicy(bytes32 policyRoot) external view returns (bool);
    function getCurrentPolicyRoot() external view returns (bytes32);
}

contract LineageVerifier is ILineageVerifier {
    
    // ============ Constants ============
    uint256 public constant MAX_DEPTH = 1_000_000;
    string public constant VERSION = "1.0.0";
    
    // ============ Immutable State ============
    IGroth16Verifier public immutable groth16Verifier;
    IPolicyRegistry public immutable policyRegistry;
    
    // ============ Mutable State ============
    address public admin;
    address public pendingAdmin;
    bool public genesisInitialized;
    bool public paused;
    bytes32 public genesisStateHash;
    bytes32 public genesisLineageCommitment;
    bytes32 public currentPolicyRoot;
    bool public allowDuplicates;
    
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
        bytes32 prevLineageCommitment;
        bytes32 newLineageCommitment;
        bytes32 policyRoot;
        bytes32 prevStateHash;
        bytes32 newStateHash;
    }
    
    struct VerificationContext {
        bytes32 proofHash;
        uint256 prevDepth;
        uint256 newDepth;
    }
    
    // ============ Events (only non-interface events) ============
    event AdminTransferInitiated(address indexed currentAdmin, address indexed pendingAdmin);
    event AdminTransferred(address indexed previousAdmin, address indexed newAdmin);
    event PolicyRootUpdated(bytes32 indexed oldRoot, bytes32 indexed newRoot);
    event PauseStatusChanged(bool isPaused);
    event GenesisReset(bytes32 indexed oldGenesisHash, bytes32 indexed newGenesisHash);
    
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
    error CannotResetGenesisAfterTransitions();
    error InvalidOriginClass();
    
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
        uint256 _genesisCommitment,
        uint256 _policyRoot,
        address _groth16Verifier,
        bool _allowDuplicates
    ) {
        if (_groth16Verifier == address(0)) revert ZeroAddress();
        
        groth16Verifier = IGroth16Verifier(_groth16Verifier);
        policyRegistry = IPolicyRegistry(address(0));
        admin = msg.sender;
        allowDuplicates = _allowDuplicates;
        currentPolicyRoot = bytes32(_policyRoot);
        genesisLineageCommitment = bytes32(_genesisCommitment);
        
        emit AdminTransferred(address(0), msg.sender);
    }
    
    // ============ View Functions for Deploy Script ============
    
    function getGenesisCommitment() external view returns (uint256) {
        return uint256(genesisLineageCommitment);
    }
    
    function getPolicyRoot() external view returns (uint256) {
        return uint256(currentPolicyRoot);
    }
    
    function getVerifierAddress() external view returns (address) {
        return address(groth16Verifier);
    }
    
    // ============ Admin Functions ============
    
    function setGenesis(
        bytes32 _genesisStateHash,
        bytes32 _genesisLineageCommitment
    ) external override onlyAdmin {
        if (genesisInitialized) revert GenesisAlreadySet();
        if (_genesisStateHash == bytes32(0)) revert ZeroStateHash();
        
        _initializeGenesis(_genesisStateHash, _genesisLineageCommitment);
        
        emit GenesisSet(_genesisStateHash, _genesisLineageCommitment, msg.sender);
    }
    
    function setGenesisWithPolicy(
        bytes32 _genesisStateHash,
        bytes32 _genesisLineageCommitment,
        bytes32 _initialPolicyRoot
    ) external onlyAdmin {
        if (genesisInitialized) revert GenesisAlreadySet();
        if (_genesisStateHash == bytes32(0)) revert ZeroStateHash();
        
        currentPolicyRoot = _initialPolicyRoot;
        _initializeGenesis(_genesisStateHash, _genesisLineageCommitment);
        
        emit GenesisSet(_genesisStateHash, _genesisLineageCommitment, msg.sender);
    }
    
    function _initializeGenesis(bytes32 _stateHash, bytes32 _lineageCommitment) internal {
        genesisStateHash = _stateHash;
        genesisLineageCommitment = _lineageCommitment;
        
        stateLineage[_stateHash] = _lineageCommitment;
        stateDepth[_stateHash] = 0;
        verifiedStates[_stateHash] = true;
        stateOriginClass[_stateHash] = 0;
        stateTimestamp[_stateHash] = block.timestamp;
        stateCreator[_stateHash] = msg.sender;
        
        genesisInitialized = true;
    }
    
    function resetGenesis(
        bytes32 _newGenesisStateHash,
        bytes32 _newGenesisLineageCommitment,
        bytes32 _newPolicyRoot
    ) external onlyAdmin {
        if (totalTransitions > 0) revert CannotResetGenesisAfterTransitions();
        if (_newGenesisStateHash == bytes32(0)) revert ZeroStateHash();
        
        bytes32 oldGenesisHash = genesisStateHash;
        
        if (genesisInitialized) {
            _clearState(genesisStateHash);
        }
        
        currentPolicyRoot = _newPolicyRoot;
        _initializeGenesis(_newGenesisStateHash, _newGenesisLineageCommitment);
        
        emit GenesisReset(oldGenesisHash, _newGenesisStateHash);
    }
    
    function _clearState(bytes32 _stateHash) internal {
        delete stateLineage[_stateHash];
        delete stateDepth[_stateHash];
        delete verifiedStates[_stateHash];
        delete stateOriginClass[_stateHash];
        delete stateTimestamp[_stateHash];
        delete stateCreator[_stateHash];
    }
    
    function updatePolicyRoot(bytes32 _newPolicyRoot) external onlyAdmin {
        bytes32 oldRoot = currentPolicyRoot;
        currentPolicyRoot = _newPolicyRoot;
        emit PolicyRootUpdated(oldRoot, _newPolicyRoot);
    }
    
    function transferAdmin(address _newAdmin) external onlyAdmin {
        if (_newAdmin == address(0)) revert ZeroAddress();
        pendingAdmin = _newAdmin;
        emit AdminTransferInitiated(admin, _newAdmin);
    }
    
    function acceptAdmin() external {
        if (msg.sender != pendingAdmin) revert NotPendingAdmin();
        address oldAdmin = admin;
        admin = pendingAdmin;
        pendingAdmin = address(0);
        emit AdminTransferred(oldAdmin, admin);
    }
    
    function setPaused(bool _paused) external onlyAdmin {
        paused = _paused;
        emit PauseStatusChanged(_paused);
    }
    
    // ============ Core Verification (2 signals - interface) ============
    
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[2] calldata publicSignals
    ) external override whenNotPaused genesisRequired returns (bool) {
        bytes32 proofHash = keccak256(abi.encodePacked(pA, pB, pC, publicSignals));
        if (usedProofs[proofHash]) revert ProofAlreadyUsed();
        
        usedProofs[proofHash] = true;
        return true;
    }
    
    // ============ Core Verification (5 signals - full) ============
    
    function verifyLineageFull(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[5] calldata publicSignals
    ) external whenNotPaused genesisRequired returns (bool) {
        PublicSignals memory signals = _parseSignals(publicSignals);
        
        VerificationContext memory ctx;
        ctx.proofHash = keccak256(abi.encodePacked(pA, pB, pC, publicSignals));
        
        if (usedProofs[ctx.proofHash]) revert ProofAlreadyUsed();
        
        _validateInputs(signals);
        
        ctx.prevDepth = stateDepth[signals.prevStateHash];
        if (ctx.prevDepth >= MAX_DEPTH) revert MaxDepthExceeded();
        ctx.newDepth = ctx.prevDepth + 1;
        
        if (!groth16Verifier.verifyProof(pA, pB, pC, publicSignals)) {
            revert InvalidProof();
        }
        
        usedProofs[ctx.proofHash] = true;
        
        _recordState(signals.newStateHash, signals.newLineageCommitment, ctx.newDepth);
        
        totalTransitions++;
        if (ctx.newDepth > maxDepthReached) {
            maxDepthReached = ctx.newDepth;
        }
        
        emit LineageVerified(
            signals.prevStateHash,
            signals.newStateHash,
            signals.newLineageCommitment,
            ctx.newDepth,
            0,
            msg.sender
        );
        
        return true;
    }
    
    function _parseSignals(uint256[5] calldata publicSignals) 
        internal 
        pure 
        returns (PublicSignals memory) 
    {
        return PublicSignals({
            prevLineageCommitment: bytes32(publicSignals[0]),
            newLineageCommitment: bytes32(publicSignals[1]),
            policyRoot: bytes32(publicSignals[2]),
            prevStateHash: bytes32(publicSignals[3]),
            newStateHash: bytes32(publicSignals[4])
        });
    }
    
    function _validateInputs(PublicSignals memory signals) internal view {
        if (signals.newStateHash == bytes32(0)) revert ZeroStateHash();
        
        if (!allowDuplicates && verifiedStates[signals.newStateHash]) {
            revert StateAlreadyExists();
        }
        
        if (!verifiedStates[signals.prevStateHash]) {
            revert PreviousStateNotVerified();
        }
        
        if (stateLineage[signals.prevStateHash] != signals.prevLineageCommitment) {
            revert LineageMismatch();
        }
        
        bytes32 expectedPolicy = _getEffectivePolicyRoot();
        if (signals.policyRoot != expectedPolicy) {
            revert PolicyMismatch();
        }
    }
    
    function _recordState(
        bytes32 _stateHash, 
        bytes32 _lineageCommitment, 
        uint256 _depth
    ) internal {
        stateLineage[_stateHash] = _lineageCommitment;
        stateDepth[_stateHash] = _depth;
        verifiedStates[_stateHash] = true;
        stateTimestamp[_stateHash] = block.timestamp;
        stateCreator[_stateHash] = msg.sender;
    }
    
    // ============ View Functions ============
    
    function getLineage(bytes32 stateHash) external view override returns (bytes32) {
        return stateLineage[stateHash];
    }
    
    function hasVerifiedLineage(bytes32 stateHash) external view override returns (bool) {
        return verifiedStates[stateHash];
    }
    
    function getDepth(bytes32 stateHash) external view override returns (uint256) {
        return stateDepth[stateHash];
    }
    
    function getEffectivePolicyRoot() external view returns (bytes32) {
        return _getEffectivePolicyRoot();
    }
    
    function isProofUsed(bytes32 proofHash) external view returns (bool) {
        return usedProofs[proofHash];
    }
    
    function _getEffectivePolicyRoot() internal view returns (bytes32) {
        if (address(policyRegistry) != address(0)) {
            return policyRegistry.getCurrentPolicyRoot();
        }
        return currentPolicyRoot;
    }
}