// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title StateRegistry (PRODUCTION)
 * @notice Track verified states and their properties
 * 
 * SECURITY:
 *  Immutable state history
 *  Lineage tracking
 *  Depth management
 *  Origin class recording
 *  Creator attribution
 *  Timestamp verification
 * 
 * PRODUCTION NOTES:
 * - Stores verified state information
 * - Enables state history queries
 * - Tracks state relationships
 * - Provides audit trail
 */

contract StateRegistry {
    
    // ============ Structures ============
    
    struct StateInfo {
        bytes32 stateHash;
        bytes32 lineageCommitment;
        uint256 depth;
        uint8 originClass;
        uint256 timestamp;
        address creator;
        bool verified;
    }
    
    struct StateTransition {
        bytes32 fromState;
        bytes32 toState;
        uint256 transitionTime;
        uint8 originClass;
    }
    
    // ============ State ============
    address public admin;
    address public immutable lineageVerifier;
    
    // State tracking
    mapping(bytes32 => StateInfo) public states;
    mapping(bytes32 => StateTransition[]) public stateTransitions;
    mapping(address => bytes32[]) public userStates;
    
    bytes32[] public allStates;
    uint256 public totalStates;
    
    // Lineage relationships
    mapping(bytes32 => bytes32) public stateParent;
    mapping(bytes32 => bytes32[]) public stateChildren;
    
    // ============ Events ============
    
    event StateRecorded(
        bytes32 indexed stateHash,
        bytes32 indexed lineageCommitment,
        uint256 depth,
        uint8 originClass,
        address creator
    );
    
    event StateLinked(
        bytes32 indexed parentState,
        bytes32 indexed childState,
        uint8 originClass
    );
    
    event StateQueried(
        bytes32 indexed stateHash,
        address indexed querier
    );
    
    event AdminTransferred(address indexed newAdmin);
    
    // ============ Errors ============
    error NotAdmin();
    error NotLineageVerifier();
    error StateNotFound();
    error StateAlreadyExists();
    error ZeroAddress();
    error InvalidStateHash();
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    modifier onlyLineageVerifier() {
        if (msg.sender != lineageVerifier) revert NotLineageVerifier();
        _;
    }
    
    // ============ Constructor ============
    constructor(address _lineageVerifier) {
        if (_lineageVerifier == address(0)) revert ZeroAddress();
        
        admin = msg.sender;
        lineageVerifier = _lineageVerifier;
        totalStates = 0;
    }
    
    // ============ Core Functions ============
    
    /**
     * @notice Record a verified state
     * 
     * Called by LineageVerifier after successful verification
     */
    function recordState(
        bytes32 stateHash,
        bytes32 lineageCommitment,
        uint256 depth,
        uint8 originClass,
        address creator
    ) external onlyLineageVerifier {
        if (stateHash == bytes32(0)) revert InvalidStateHash();
        if (states[stateHash].verified) revert StateAlreadyExists();
        
        StateInfo memory stateInfo = StateInfo({
            stateHash: stateHash,
            lineageCommitment: lineageCommitment,
            depth: depth,
            originClass: originClass,
            timestamp: block.timestamp,
            creator: creator,
            verified: true
        });
        
        states[stateHash] = stateInfo;
        allStates.push(stateHash);
        userStates[creator].push(stateHash);
        totalStates++;
        
        emit StateRecorded(
            stateHash,
            lineageCommitment,
            depth,
            originClass,
            creator
        );
    }
    
    /**
     * @notice Link parent and child states
     */
    function linkStates(
        bytes32 parentState,
        bytes32 childState,
        uint8 originClass
    ) external onlyLineageVerifier {
        if (!states[parentState].verified) revert StateNotFound();
        if (!states[childState].verified) revert StateNotFound();
        
        stateParent[childState] = parentState;
        stateChildren[parentState].push(childState);
        
        StateTransition memory transition = StateTransition({
            fromState: parentState,
            toState: childState,
            transitionTime: block.timestamp,
            originClass: originClass
        });
        
        stateTransitions[parentState].push(transition);
        
        emit StateLinked(parentState, childState, originClass);
    }
    
    // ============ Query Functions ============
    
    /**
     * @notice Get state information
     */
    function getState(bytes32 stateHash) 
        external returns (StateInfo memory) 
    {
        if (!states[stateHash].verified) revert StateNotFound();
        
        emit StateQueried(stateHash, msg.sender);
        return states[stateHash];
    }
    
    /**
     * @notice Check if state is verified
     */
    function isStateVerified(bytes32 stateHash) 
        external view returns (bool) 
    {
        return states[stateHash].verified;
    }
    
    /**
     * @notice Get state depth
     */
    function getStateDepth(bytes32 stateHash) 
        external view returns (uint256) 
    {
        if (!states[stateHash].verified) revert StateNotFound();
        return states[stateHash].depth;
    }
    
    /**
     * @notice Get state origin class
     */
    function getStateOriginClass(bytes32 stateHash) 
        external view returns (uint8) 
    {
        if (!states[stateHash].verified) revert StateNotFound();
        return states[stateHash].originClass;
    }
    
    /**
     * @notice Get parent state
     */
    function getParentState(bytes32 stateHash) 
        external view returns (bytes32) 
    {
        if (!states[stateHash].verified) revert StateNotFound();
        return stateParent[stateHash];
    }
    
    /**
     * @notice Get child states
     */
    function getChildStates(bytes32 stateHash) 
        external view returns (bytes32[] memory) 
    {
        if (!states[stateHash].verified) revert StateNotFound();
        return stateChildren[stateHash];
    }
    
    /**
     * @notice Get state transitions
     */
    function getTransitions(bytes32 stateHash) 
        external view returns (StateTransition[] memory) 
    {
        if (!states[stateHash].verified) revert StateNotFound();
        return stateTransitions[stateHash];
    }
    
    /**
     * @notice Get all states for user
     */
    function getUserStates(address user) 
        external view returns (bytes32[] memory) 
    {
        return userStates[user];
    }
    
    /**
     * @notice Get all verified states
     */
    function getAllStates() 
        external view returns (bytes32[] memory) 
    {
        return allStates;
    }
    
    /**
     * @notice Get total states
     */
    function getTotalStates() 
        external view returns (uint256) 
    {
        return totalStates;
    }
    
    /**
     * @notice Trace lineage (get all ancestors)
     */
    function traceLineage(bytes32 stateHash) 
        external view returns (bytes32[] memory ancestors) 
    {
        if (!states[stateHash].verified) revert StateNotFound();
        
        bytes32[] memory ancestorList = new bytes32[](states[stateHash].depth + 1);
        bytes32 current = stateHash;
        uint256 index = 0;
        
        while (current != bytes32(0) && index <= states[stateHash].depth) {
            ancestorList[index] = current;
            current = stateParent[current];
            index++;
        }
        
        return ancestorList;
    }
    
    // ============ Analytics Functions ============
    
    /**
     * @notice Get state creation statistics
     */
    function getStatistics() 
        external view returns (
            uint256 totalVerifiedStates,
            uint256 averageDepth,
            uint256 maxDepth
        ) 
    {
        totalVerifiedStates = totalStates;
        
        uint256 totalDepth = 0;
        uint256 max = 0;
        
        for (uint256 i = 0; i < allStates.length; i++) {
            uint256 depth = states[allStates[i]].depth;
            totalDepth += depth;
            if (depth > max) {
                max = depth;
            }
        }
        
        averageDepth = totalStates > 0 ? totalDepth / totalStates : 0;
        maxDepth = max;
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Transfer admin role
     */
    function transferAdmin(address newAdmin) external onlyAdmin {
        if (newAdmin == address(0)) revert ZeroAddress();
        admin = newAdmin;
        emit AdminTransferred(newAdmin);
    }
}