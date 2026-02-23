// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/ILineageVerifier.sol";

/**
 * @title Groth16Verifier Interface
 * @notice Interface for the snarkjs-generated Groth16 verifier
 */
interface IGroth16Verifier {
    function verifyProof(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[5] calldata pubSignals
    ) external view returns (bool);
}

/**
 * @title PolicyRegistry Interface
 * @notice Interface for policy management
 */
interface IPolicyRegistry {
    function isValidPolicy(bytes32 policyRoot) external view returns (bool);
    function getCurrentPolicyRoot() external view returns (bytes32);
}

/**
 * @title LineageVerifier
 * @author ZK-ORIGIN Team
 * @notice Verifies and tracks state lineage proofs on-chain
 * @dev This contract maintains a mapping of state hashes to their lineage commitments
 * 
 * ## Architecture Overview
 * 
 * The LineageVerifier works in conjunction with:
 * 1. Groth16Verifier - Cryptographic proof verification
 * 2. PolicyRegistry - Origin policy management
 * 
 * ## State Lineage Model
 * 
 * Each state has:
 * - stateHash: Unique identifier for the state
 * - lineageCommitment: Cryptographic commitment to entire history
 * - depth: Number of transitions from genesis
 * - originClass: Type of entity that created this state
 * 
 * ## Security Properties
 * 
 * 1. Soundness: Only valid ZK proofs can register new states
 * 2. Lineage Integrity: Each state must descend from verified predecessor
 * 3. Policy Compliance: All transitions must follow origin policy
 * 4. Non-replayability: Each proof can only be used once
 * 
 * ## Public Signals Layout (5 signals)
 * 
 * [0] prev_lineage_commitment - Lineage commitment of previous state
 * [1] new_lineage_commitment  - Lineage commitment of new state
 * [2] policy_root             - Merkle root of origin policy
 * [3] prev_state_hash         - Hash of previous state
 * [4] new_state_hash          - Hash of new state
 */
abstract contract LineageVerifier is ILineageVerifier {
    
    // ============ Constants ============
    
    /// @notice Maximum allowed lineage depth (prevent DoS)
    uint256 public constant MAX_DEPTH = 1_000_000;
    
    /// @notice Minimum time between transitions for same origin (rate limiting)
    uint256 public constant MIN_TRANSITION_INTERVAL = 1; // 1 second
    
    /// @notice Version of the contract
    string public constant VERSION = "1.0.0";
    
    // ============ Immutable State ============
    
    /// @notice The Groth16 verifier contract
    IGroth16Verifier public immutable groth16Verifier;
    
    /// @notice The policy registry contract (optional, can be address(0))
    IPolicyRegistry public immutable policyRegistry;
    
    // ============ Mutable State ============
    
    /// @notice Admin address (can set genesis and update settings)
    address public admin;
    
    /// @notice Pending admin for two-step transfer
    address public pendingAdmin;
    
    /// @notice Whether genesis has been initialized
    bool public genesisInitialized;
    
    /// @notice Whether the contract is paused
    bool public paused;
    
    /// @notice Genesis state hash
    bytes32 public genesisStateHash;
    
    /// @notice Genesis lineage commitment
    bytes32 public genesisLineageCommitment;
    
    /// @notice Current policy root (if not using PolicyRegistry)
    bytes32 public currentPolicyRoot;
    
    /// @notice Mapping from state hash to lineage commitment
    mapping(bytes32 => bytes32) public stateLineage;
    
    /// @notice Mapping from state hash to lineage depth
    mapping(bytes32 => uint256) public stateDepth;
    
    /// @notice Mapping to track which states have been verified
    mapping(bytes32 => bool) public verifiedStates;
    
    /// @notice Mapping from state hash to origin class
    mapping(bytes32 => uint8) public stateOriginClass;
    
    /// @notice Mapping from state hash to creation timestamp
    mapping(bytes32 => uint256) public stateTimestamp;
    
    /// @notice Mapping from state hash to creator address
    mapping(bytes32 => address) public stateCreator;
    
    /// @notice Mapping to track used proof hashes (prevent replay)
    mapping(bytes32 => bool) public usedProofs;
    
    /// @notice Total number of verified transitions
    uint256 public totalTransitions;
    
    /// @notice Maximum depth reached
    uint256 public maxDepthReached;
    
    // ============ Events ============
    
    /// @notice Emitted when genesis is set
    event GenesisSet(
        bytes32 indexed genesisStateHash,
        bytes32 indexed genesisLineageCommitment,
        address indexed setter
    );
    
    /// @notice Emitted when a lineage proof is verified
    event LineageVerified(
        bytes32 indexed prevStateHash,
        bytes32 indexed newStateHash,
        bytes32 lineageCommitment,
        uint256 depth,
        uint8 originClass,
        address indexed creator
    );
    
    /// @notice Emitted when admin transfer is initiated
    event AdminTransferInitiated(address indexed currentAdmin, address indexed pendingAdmin);
    
    /// @notice Emitted when admin transfer is completed
    event AdminTransferred(address indexed previousAdmin, address indexed newAdmin);
    
    /// @notice Emitted when policy root is updated
    event PolicyRootUpdated(bytes32 indexed oldRoot, bytes32 indexed newRoot);
    
    /// @notice Emitted when contract is paused/unpaused
    event PauseStatusChanged(bool isPaused);
    
    /// @notice Emitted when genesis is reset (only before any transitions)
    event GenesisReset(
        bytes32 indexed oldGenesisHash,
        bytes32 indexed newGenesisHash
    );
    
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
    error InvalidPublicSignals();
    error StateAlreadyExists();
    error CannotResetGenesisAfterTransitions();
    error InvalidOriginClass();
    error RateLimitExceeded();
    
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
    
    /**
     * @notice Deploy the LineageVerifier
     * @param _groth16Verifier Address of the Groth16 verifier contract
     * @param _policyRegistry Address of the policy registry (can be address(0))
     */
    constructor(address _groth16Verifier, address _policyRegistry) {
        if (_groth16Verifier == address(0)) revert ZeroAddress();
        
        groth16Verifier = IGroth16Verifier(_groth16Verifier);
        policyRegistry = IPolicyRegistry(_policyRegistry);
        admin = msg.sender;
        
        emit AdminTransferred(address(0), msg.sender);
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Set the genesis state
     * @param _genesisStateHash Hash of the genesis state
     * @param _genesisLineageCommitment Initial lineage commitment
     * @param _initialPolicyRoot Initial policy root (if not using PolicyRegistry)
     */
    function setGenesis(
        bytes32 _genesisStateHash,
        bytes32 _genesisLineageCommitment,
        bytes32 _initialPolicyRoot
    ) external onlyAdmin {
        if (genesisInitialized) revert GenesisAlreadySet();
        if (_genesisStateHash == bytes32(0)) revert ZeroStateHash();
        
        genesisStateHash = _genesisStateHash;
        genesisLineageCommitment = _genesisLineageCommitment;
        currentPolicyRoot = _initialPolicyRoot;
        
        // Register genesis state
        stateLineage[_genesisStateHash] = _genesisLineageCommitment;
        stateDepth[_genesisStateHash] = 0;
        verifiedStates[_genesisStateHash] = true;
        stateOriginClass[_genesisStateHash] = 0; // Genesis origin
        stateTimestamp[_genesisStateHash] = block.timestamp;
        stateCreator[_genesisStateHash] = msg.sender;
        
        genesisInitialized = true;
        
        emit GenesisSet(_genesisStateHash, _genesisLineageCommitment, msg.sender);
    }
    
    /**
     * @notice Reset genesis (only allowed before any transitions)
     * @param _newGenesisStateHash New genesis state hash
     * @param _newGenesisLineageCommitment New genesis lineage commitment
     * @param _newPolicyRoot New policy root
     */
    function resetGenesis(
        bytes32 _newGenesisStateHash,
        bytes32 _newGenesisLineageCommitment,
        bytes32 _newPolicyRoot
    ) external onlyAdmin {
        if (totalTransitions > 0) revert CannotResetGenesisAfterTransitions();
        if (_newGenesisStateHash == bytes32(0)) revert ZeroStateHash();
        
        bytes32 oldGenesisHash = genesisStateHash;
        
        // Clear old genesis
        if (genesisInitialized) {
            delete stateLineage[genesisStateHash];
            delete stateDepth[genesisStateHash];
            delete verifiedStates[genesisStateHash];
            delete stateOriginClass[genesisStateHash];
            delete stateTimestamp[genesisStateHash];
            delete stateCreator[genesisStateHash];
        }
        
        // Set new genesis
        genesisStateHash = _newGenesisStateHash;
        genesisLineageCommitment = _newGenesisLineageCommitment;
        currentPolicyRoot = _newPolicyRoot;
        
        stateLineage[_newGenesisStateHash] = _newGenesisLineageCommitment;
        stateDepth[_newGenesisStateHash] = 0;
        verifiedStates[_newGenesisStateHash] = true;
        stateOriginClass[_newGenesisStateHash] = 0;
        stateTimestamp[_newGenesisStateHash] = block.timestamp;
        stateCreator[_newGenesisStateHash] = msg.sender;
        
        genesisInitialized = true;
        
        emit GenesisReset(oldGenesisHash, _newGenesisStateHash);
    }
    
    /**
     * @notice Update the policy root
     * @param _newPolicyRoot New policy Merkle root
     */
    function updatePolicyRoot(bytes32 _newPolicyRoot) external onlyAdmin {
        bytes32 oldRoot = currentPolicyRoot;
        currentPolicyRoot = _newPolicyRoot;
        emit PolicyRootUpdated(oldRoot, _newPolicyRoot);
    }
    
    /**
     * @notice Initiate admin transfer (two-step process)
     * @param _newAdmin Address of the new admin
     */
    function transferAdmin(address _newAdmin) external onlyAdmin {
        if (_newAdmin == address(0)) revert ZeroAddress();
        pendingAdmin = _newAdmin;
        emit AdminTransferInitiated(admin, _newAdmin);
    }
    
    /**
     * @notice Accept admin transfer
     */
    function acceptAdmin() external {
        if (msg.sender != pendingAdmin) revert NotPendingAdmin();
        address oldAdmin = admin;
        admin = pendingAdmin;
        pendingAdmin = address(0);
        emit AdminTransferred(oldAdmin, admin);
    }
    
    /**
     * @notice Pause/unpause the contract
     * @param _paused New pause status
     */
    function setPaused(bool _paused) external onlyAdmin {
        paused = _paused;
        emit PauseStatusChanged(_paused);
    }
    
    // ============ Core Verification Functions ============
    
    /**
     * @notice Verify and record a lineage proof
     * @param pA Groth16 proof element A (2 elements)
     * @param pB Groth16 proof element B (2x2 elements)
     * @param pC Groth16 proof element C (2 elements)
     * @param publicSignals Public signals array (5 elements)
     *        [0] prev_lineage_commitment
     *        [1] new_lineage_commitment
     *        [2] policy_root
     *        [3] prev_state_hash
     *        [4] new_state_hash
     * @return success Whether the proof was valid and recorded
     */
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[5] calldata publicSignals
    ) external whenNotPaused genesisRequired returns (bool success) {
        // Compute proof hash for replay protection
        bytes32 proofHash = keccak256(abi.encodePacked(pA, pB, pC, publicSignals));
        if (usedProofs[proofHash]) revert ProofAlreadyUsed();
        
        // Extract public signals
        bytes32 prevLineageCommitment = bytes32(publicSignals[0]);
        bytes32 newLineageCommitment = bytes32(publicSignals[1]);
        bytes32 proofPolicyRoot = bytes32(publicSignals[2]);
        bytes32 prevStateHash = bytes32(publicSignals[3]);
        bytes32 newStateHash = bytes32(publicSignals[4]);
        
        // Validate inputs
        if (newStateHash == bytes32(0)) revert ZeroStateHash();
        if (verifiedStates[newStateHash]) revert StateAlreadyExists();
        
        // Verify previous state exists and matches
        if (!verifiedStates[prevStateHash]) revert PreviousStateNotVerified();
        if (stateLineage[prevStateHash] != prevLineageCommitment) revert LineageMismatch();
        
        // Verify policy root matches current policy
        bytes32 expectedPolicyRoot = _getEffectivePolicyRoot();
        if (proofPolicyRoot != expectedPolicyRoot) revert PolicyMismatch();
        
        // Check depth limit
        uint256 prevDepth = stateDepth[prevStateHash];
        if (prevDepth >= MAX_DEPTH) revert MaxDepthExceeded();
        
        // Verify the ZK proof
        bool proofValid = groth16Verifier.verifyProof(pA, pB, pC, publicSignals);
        if (!proofValid) revert InvalidProof();
        
        // Mark proof as used
        usedProofs[proofHash] = true;
        
        // Calculate new depth
        uint256 newDepth = prevDepth + 1;
        
        // Record the new state
        stateLineage[newStateHash] = newLineageCommitment;
        stateDepth[newStateHash] = newDepth;
        verifiedStates[newStateHash] = true;
        stateTimestamp[newStateHash] = block.timestamp;
        stateCreator[newStateHash] = msg.sender;
        
        // Update statistics
        totalTransitions++;
        if (newDepth > maxDepthReached) {
            maxDepthReached = newDepth;
        }
        
        emit LineageVerified(
            prevStateHash,
            newStateHash,
            newLineageCommitment,
            newDepth,
            0, // Origin class extracted from circuit if needed
            msg.sender
        );
        
        return true;
    }
    
    /**
     * @notice Verify lineage with explicit origin class
     * @dev Use this when origin class is a public signal in the circuit
     */
    function verifyLineageWithOrigin(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[5] calldata publicSignals,
        uint8 originClass
    ) external whenNotPaused genesisRequired returns (bool success) {
        // Validate origin class
        if (originClass > 6) revert InvalidOriginClass();
        
        // Compute proof hash for replay protection
        bytes32 proofHash = keccak256(abi.encodePacked(pA, pB, pC, publicSignals, originClass));
        if (usedProofs[proofHash]) revert ProofAlreadyUsed();
        
        // Extract public signals
        bytes32 prevLineageCommitment = bytes32(publicSignals[0]);
        bytes32 newLineageCommitment = bytes32(publicSignals[1]);
        bytes32 proofPolicyRoot = bytes32(publicSignals[2]);
        bytes32 prevStateHash = bytes32(publicSignals[3]);
        bytes32 newStateHash = bytes32(publicSignals[4]);
        
        // Validate inputs
        if (newStateHash == bytes32(0)) revert ZeroStateHash();
        if (verifiedStates[newStateHash]) revert StateAlreadyExists();
        
        // Verify previous state exists and matches
        if (!verifiedStates[prevStateHash]) revert PreviousStateNotVerified();
        if (stateLineage[prevStateHash] != prevLineageCommitment) revert LineageMismatch();
        
        // Verify policy root
        bytes32 expectedPolicyRoot = _getEffectivePolicyRoot();
        if (proofPolicyRoot != expectedPolicyRoot) revert PolicyMismatch();
        
        // Check depth limit
        uint256 prevDepth = stateDepth[prevStateHash];
        if (prevDepth >= MAX_DEPTH) revert MaxDepthExceeded();
        
        // Verify the ZK proof
        bool proofValid = groth16Verifier.verifyProof(pA, pB, pC, publicSignals);
        if (!proofValid) revert InvalidProof();
        
        // Mark proof as used
        usedProofs[proofHash] = true;
        
        // Calculate new depth
        uint256 newDepth = prevDepth + 1;
        
        // Record the new state
        stateLineage[newStateHash] = newLineageCommitment;
        stateDepth[newStateHash] = newDepth;
        verifiedStates[newStateHash] = true;
        stateOriginClass[newStateHash] = originClass;
        stateTimestamp[newStateHash] = block.timestamp;
        stateCreator[newStateHash] = msg.sender;
        
        // Update statistics
        totalTransitions++;
        if (newDepth > maxDepthReached) {
            maxDepthReached = newDepth;
        }
        
        emit LineageVerified(
            prevStateHash,
            newStateHash,
            newLineageCommitment,
            newDepth,
            originClass,
            msg.sender
        );
        
        return true;
    }
    
    /**
     * @notice Batch verify multiple lineage proofs
     * @dev More gas efficient for sequential transitions
     * @param proofs Array of proof data
     * @return success Whether all proofs were valid
     */
    function verifyLineageBatch(
        ProofData[] calldata proofs
    ) external whenNotPaused genesisRequired returns (bool success) {
        uint256 length = proofs.length;
        require(length > 0, "Empty batch");
        require(length <= 50, "Batch too large");
        
        for (uint256 i = 0; i < length; i++) {
            bool result = _verifyLineageInternal(
                proofs[i].pA,
                proofs[i].pB,
                proofs[i].pC,
                proofs[i].publicSignals
            );
            require(result, "Proof failed");
        }
        
        return true;
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Get the lineage commitment for a state
     * @param stateHash Hash of the state
     * @return The lineage commitment
     */
    function getLineage(bytes32 stateHash) external view returns (bytes32) {
        return stateLineage[stateHash];
    }
    
    /**
     * @notice Check if a state has verified lineage
     * @param stateHash Hash of the state
     * @return Whether the state has verified lineage
     */
    function hasVerifiedLineage(bytes32 stateHash) external view returns (bool) {
        return verifiedStates[stateHash];
    }
    
    /**
     * @notice Get the lineage depth for a state
     * @param stateHash Hash of the state
     * @return The lineage depth
     */
    function getDepth(bytes32 stateHash) external view returns (uint256) {
        return stateDepth[stateHash];
    }
    
    /**
     * @notice Get full state information
     * @param stateHash Hash of the state
     * @return info StateInfo struct with all state data
     */
    function getStateInfo(bytes32 stateHash) external view returns (StateInfo memory info) {
        return StateInfo({
            lineageCommitment: stateLineage[stateHash],
            depth: stateDepth[stateHash],
            verified: verifiedStates[stateHash],
            originClass: stateOriginClass[stateHash],
            timestamp: stateTimestamp[stateHash],
            creator: stateCreator[stateHash]
        });
    }
    
    /**
     * @notice Get multiple state infos at once
     * @param stateHashes Array of state hashes
     * @return infos Array of StateInfo structs
     */
    function getStateInfoBatch(bytes32[] calldata stateHashes) 
        external 
        view 
        returns (StateInfo[] memory infos) 
    {
        uint256 length = stateHashes.length;
        infos = new StateInfo[](length);
        
        for (uint256 i = 0; i < length; i++) {
            bytes32 stateHash = stateHashes[i];
            infos[i] = StateInfo({
                lineageCommitment: stateLineage[stateHash],
                depth: stateDepth[stateHash],
                verified: verifiedStates[stateHash],
                originClass: stateOriginClass[stateHash],
                timestamp: stateTimestamp[stateHash],
                creator: stateCreator[stateHash]
            });
        }
    }
    
    /**
     * @notice Get current effective policy root
     * @return The policy root currently in effect
     */
    function getEffectivePolicyRoot() external view returns (bytes32) {
        return _getEffectivePolicyRoot();
    }
    
    /**
     * @notice Get contract statistics
     * @return stats ContractStats struct
     */
    function getStats() external view returns (ContractStats memory stats) {
        return ContractStats({
            totalTransitions: totalTransitions,
            maxDepthReached: maxDepthReached,
            genesisInitialized: genesisInitialized,
            paused: paused,
            genesisStateHash: genesisStateHash,
            currentPolicyRoot: _getEffectivePolicyRoot()
        });
    }
    
    /**
     * @notice Check if a proof has been used
     * @param proofHash Hash of the proof
     * @return Whether the proof has been used
     */
    function isProofUsed(bytes32 proofHash) external view returns (bool) {
        return usedProofs[proofHash];
    }
    
    /**
     * @notice Compute proof hash for a given proof
     * @dev Useful for checking if a proof will be rejected as replay
     */
    function computeProofHash(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[5] calldata publicSignals
    ) external pure returns (bytes32) {
        return keccak256(abi.encodePacked(pA, pB, pC, publicSignals));
    }
    
    // ============ Internal Functions ============
    
    /**
     * @notice Internal lineage verification logic
     */
    function _verifyLineageInternal(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[5] calldata publicSignals
    ) internal returns (bool) {
        bytes32 proofHash = keccak256(abi.encodePacked(pA, pB, pC, publicSignals));
        if (usedProofs[proofHash]) revert ProofAlreadyUsed();
        
        bytes32 prevLineageCommitment = bytes32(publicSignals[0]);
        bytes32 newLineageCommitment = bytes32(publicSignals[1]);
        bytes32 proofPolicyRoot = bytes32(publicSignals[2]);
        bytes32 prevStateHash = bytes32(publicSignals[3]);
        bytes32 newStateHash = bytes32(publicSignals[4]);
        
        if (newStateHash == bytes32(0)) revert ZeroStateHash();
        if (verifiedStates[newStateHash]) revert StateAlreadyExists();
        if (!verifiedStates[prevStateHash]) revert PreviousStateNotVerified();
        if (stateLineage[prevStateHash] != prevLineageCommitment) revert LineageMismatch();
        
        bytes32 expectedPolicyRoot = _getEffectivePolicyRoot();
        if (proofPolicyRoot != expectedPolicyRoot) revert PolicyMismatch();
        
        uint256 prevDepth = stateDepth[prevStateHash];
        if (prevDepth >= MAX_DEPTH) revert MaxDepthExceeded();
        
        bool proofValid = groth16Verifier.verifyProof(pA, pB, pC, publicSignals);
        if (!proofValid) revert InvalidProof();
        
        usedProofs[proofHash] = true;
        
        uint256 newDepth = prevDepth + 1;
        
        stateLineage[newStateHash] = newLineageCommitment;
        stateDepth[newStateHash] = newDepth;
        verifiedStates[newStateHash] = true;
        stateTimestamp[newStateHash] = block.timestamp;
        stateCreator[newStateHash] = msg.sender;
        
        totalTransitions++;
        if (newDepth > maxDepthReached) {
            maxDepthReached = newDepth;
        }
        
        emit LineageVerified(
            prevStateHash,
            newStateHash,
            newLineageCommitment,
            newDepth,
            0,
            msg.sender
        );
        
        return true;
    }
    
    /**
     * @notice Get the effective policy root (from registry or local)
     */
    function _getEffectivePolicyRoot() internal view returns (bytes32) {
        if (address(policyRegistry) != address(0)) {
            return policyRegistry.getCurrentPolicyRoot();
        }
        return currentPolicyRoot;
    }
    
    // ============ Structs ============
    
    struct ProofData {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[5] publicSignals;
    }
    
    struct StateInfo {
        bytes32 lineageCommitment;
        uint256 depth;
        bool verified;
        uint8 originClass;
        uint256 timestamp;
        address creator;
    }
    
    struct ContractStats {
        uint256 totalTransitions;
        uint256 maxDepthReached;
        bool genesisInitialized;
        bool paused;
        bytes32 genesisStateHash;
        bytes32 currentPolicyRoot;
    }
}