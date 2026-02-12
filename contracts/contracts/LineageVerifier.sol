// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/ILineageVerifier.sol";

/**
 * @title Groth16Verifier
 * @notice Auto-generated Groth16 verifier - this will be replaced by snarkjs export
 * @dev This is a placeholder. Run circuits/scripts/setup.sh to generate the real one.
 */
interface IGroth16Verifier {
    function verifyProof(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[2] calldata pubSignals
    ) external view returns (bool);
}

/**
 * @title LineageVerifier
 * @author ZK-ORIGIN Team
 * @notice Verifies and tracks state lineage proofs on-chain
 * @dev This contract maintains a mapping of state hashes to their lineage commitments
 * 
 * ## How It Works
 * 
 * 1. Genesis Setup:
 *    - Admin calls setGenesis() with the initial state hash and lineage commitment
 *    - This establishes the root of the lineage tree
 * 
 * 2. State Transitions:
 *    - For each new state, user generates a ZK proof off-chain
 *    - Proof demonstrates:
 *      a) The transition follows origin policy
 *      b) The lineage commitment is correctly updated
 *    - User calls verifyLineage() with the proof
 *    - Contract verifies proof and records the new lineage
 * 
 * 3. Verification:
 *    - Anyone can call hasVerifiedLineage() to check if a state has valid lineage
 *    - This enables trustless verification of state provenance
 */
contract LineageVerifier is ILineageVerifier {
    // State Variables
    
    /// @notice The Groth16 verifier contract
    IGroth16Verifier public immutable groth16Verifier;
    
    /// @notice Admin address (can set genesis)
    address public admin;
    
    /// @notice Whether genesis has been set
    bool public genesisInitialized;
    
    /// @notice Genesis state hash
    bytes32 public genesisStateHash;
    
    /// @notice Genesis lineage commitment
    bytes32 public genesisLineageCommitment;
    
    /// @notice Mapping from state hash to lineage commitment
    mapping(bytes32 => bytes32) public stateLineage;
    
    /// @notice Mapping from state hash to lineage depth
    mapping(bytes32 => uint256) public stateDepth;
    
    /// @notice Mapping to track which states have been verified
    mapping(bytes32 => bool) public verifiedStates;
    
    /// @notice Total number of verified transitions
    uint256 public totalTransitions;
    
    // Errors
    
    error NotAdmin();
    error GenesisAlreadySet();
    error GenesisNotSet();
    error InvalidProof();
    error PreviousStateNotVerified();
    error LineageMismatch();
    error ZeroStateHash();
    
    // Modifiers
    
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    modifier genesisRequired() {
        if (!genesisInitialized) revert GenesisNotSet();
        _;
    }
    
    // Constructor
    
    /**
     * @notice Deploy the LineageVerifier
     * @param _groth16Verifier Address of the Groth16 verifier contract
     */
    constructor(address _groth16Verifier) {
        groth16Verifier = IGroth16Verifier(_groth16Verifier);
        admin = msg.sender;
    }
    // Admin Functions
    
    /**
     * @notice Set the genesis state
     * @param _genesisStateHash Hash of the genesis state
     * @param _genesisLineageCommitment Initial lineage commitment
     */
    function setGenesis(
        bytes32 _genesisStateHash,
        bytes32 _genesisLineageCommitment
    ) external onlyAdmin {
        if (genesisInitialized) revert GenesisAlreadySet();
        if (_genesisStateHash == bytes32(0)) revert ZeroStateHash();
        
        genesisStateHash = _genesisStateHash;
        genesisLineageCommitment = _genesisLineageCommitment;
        
        stateLineage[_genesisStateHash] = _genesisLineageCommitment;
        stateDepth[_genesisStateHash] = 0;
        verifiedStates[_genesisStateHash] = true;
        
        genesisInitialized = true;
        
        emit GenesisSet(_genesisStateHash, _genesisLineageCommitment);
    }
    
    /**
     * @notice Transfer admin role
     * @param newAdmin Address of the new admin
     */
    function transferAdmin(address newAdmin) external onlyAdmin {
        admin = newAdmin;
    }
    
    // Core Functions
    
    /**
     * @notice Verify and record a lineage proof
     * @param pA Proof element A
     * @param pB Proof element B
     * @param pC Proof element C
     * @param publicSignals Public signals [new_lineage_commitment, new_depth]
     * @return success Whether the proof was valid and recorded
     * 
     * @dev The proof must demonstrate:
     * 1. Valid transition from a previous verified state
     * 2. Origin policy compliance
     * 3. Correct lineage commitment update
     * 
     * Note: In the simplified circuit, we only have 2 public outputs.
     * The full circuit would include prev_state_hash, new_state_hash, prev_lineage_commitment.
     */
    function verifyLineage(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[2] calldata publicSignals
    ) external genesisRequired returns (bool success) {
        // Verify the ZK proof
        bool proofValid = groth16Verifier.verifyProof(pA, pB, pC, publicSignals);
        if (!proofValid) revert InvalidProof();
        
        // Extract public signals
        bytes32 newLineageCommitment = bytes32(publicSignals[0]);
        uint256 newDepth = publicSignals[1];
        
        // For the simplified version, we use newLineageCommitment as state identifier
        // In production, you'd have separate state hash in public signals
        bytes32 newStateHash = newLineageCommitment; // Simplified
        
        // Record the new state
        stateLineage[newStateHash] = newLineageCommitment;
        stateDepth[newStateHash] = newDepth;
        verifiedStates[newStateHash] = true;
        totalTransitions++;
        
        emit LineageVerified(
            bytes32(0), // prev state hash not in simplified circuit
            newStateHash,
            newLineageCommitment,
            newDepth
        );
        
        return true;
    }
    
    /**
     * @notice Verify lineage with explicit state hashes
     * @dev Use this when you have the full circuit with state hashes as public inputs
     */
    function verifyLineageWithStates(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        bytes32 prevStateHash,
        bytes32 newStateHash,
        bytes32 prevLineageCommitment,
        uint256[2] calldata publicSignals // [new_lineage, new_depth]
    ) external genesisRequired returns (bool success) {
        // Verify previous state has valid lineage
        if (!verifiedStates[prevStateHash]) revert PreviousStateNotVerified();
        
        // Verify previous lineage commitment matches
        if (stateLineage[prevStateHash] != prevLineageCommitment) revert LineageMismatch();
        
        // Verify the ZK proof
        bool proofValid = groth16Verifier.verifyProof(pA, pB, pC, publicSignals);
        if (!proofValid) revert InvalidProof();
        
        // Extract outputs
        bytes32 newLineageCommitment = bytes32(publicSignals[0]);
        uint256 newDepth = publicSignals[1];
        
        // Record the new state
        stateLineage[newStateHash] = newLineageCommitment;
        stateDepth[newStateHash] = newDepth;
        verifiedStates[newStateHash] = true;
        totalTransitions++;
        
        emit LineageVerified(
            prevStateHash,
            newStateHash,
            newLineageCommitment,
            newDepth
        );
        
        return true;
    }
    
    // View Functions
    
    /**
     * @notice Get the lineage commitment for a state
     * @param stateHash Hash of the state
     * @return lineageCommitment The lineage commitment
     */
    function getLineage(bytes32 stateHash) external view returns (bytes32) {
        return stateLineage[stateHash];
    }
    
    /**
     * @notice Check if a state has verified lineage
     * @param stateHash Hash of the state
     * @return verified Whether the state has verified lineage
     */
    function hasVerifiedLineage(bytes32 stateHash) external view returns (bool) {
        return verifiedStates[stateHash];
    }
    
    /**
     * @notice Get the lineage depth for a state
     * @param stateHash Hash of the state
     * @return depth The lineage depth
     */
    function getDepth(bytes32 stateHash) external view returns (uint256) {
        return stateDepth[stateHash];
    }
    
    /**
     * @notice Get full state info
     * @param stateHash Hash of the state
     * @return lineageCommitment The lineage commitment
     * @return depth The lineage depth
     * @return verified Whether the state is verified
     */
    function getStateInfo(bytes32 stateHash) external view returns (
        bytes32 lineageCommitment,
        uint256 depth,
        bool verified
    ) {
        return (
            stateLineage[stateHash],
            stateDepth[stateHash],
            verifiedStates[stateHash]
        );
    }
}