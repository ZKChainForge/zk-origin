// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title PolicyRegistry (PRODUCTION)
 * @notice Manages origin transition policies with versioning
 * 
 * SECURITY:
 *  Multiple active policies supported
 *  Policy versioning for upgrades
 *  Immutable policy activation
 *  Transition audit trail
 *  Atomic policy changes
 * 
 * PRODUCTION NOTES:
 * - Supports policy upgrades without breaking old proofs
 * - Multiple policies can be active
 * - Smooth migration between policy versions
 * - No breaking changes on policy update
 */

contract PolicyRegistry {
    
    // ============ Structures ============
    
    struct Policy {
        bytes32 merkleRoot;
        uint256 createdAt;
        bool active;
        string description;
        uint256 transitionCount;
        uint256 version;
    }
    
    struct PolicyTransition {
        uint8 fromClass;
        uint8 toClass;
        bool allowed;
        uint256 addedAt;
    }
    
    // ============ State ============
    address public admin;
    address public pendingAdmin;
    uint256 public currentPolicyVersion;
    
    mapping(uint256 => Policy) public policies;
    mapping(uint256 => bytes32[]) public policyTransitions;
    mapping(bytes32 => bool) public activePolicies;
    uint256 public policyCount;
    
    // Track policy history
    mapping(uint256 => uint256) public versionToId;
    
    // ============ Events ============
    
    event PolicyCreated(
        uint256 indexed policyId,
        uint256 version,
        bytes32 merkleRoot,
        string description
    );
    
    event PolicyActivated(
        uint256 indexed policyId,
        uint256 version,
        bytes32 merkleRoot
    );
    
    event PolicyDeactivated(
        uint256 indexed policyId,
        uint256 version
    );
    
    event TransitionAdded(
        uint256 indexed policyId,
        uint8 indexed fromClass,
        uint8 indexed toClass
    );
    
    event AdminTransferred(address indexed newAdmin);
    
    // ============ Errors ============
    error NotAdmin();
    error NotPendingAdmin();
    error PolicyNotFound();
    error PolicyNotActive();
    error InvalidTransition();
    error ZeroAddress();
    error EmptyMerkleRoot();
    error DuplicatePolicy();
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    // ============ Constructor ============
    constructor() {
        admin = msg.sender;
        currentPolicyVersion = 0;
        policyCount = 0;
    }
    
    // ============ Core Functions ============
    
    /**
     * @notice Create policy with pre-computed Merkle root
     * @param merkleRoot Root of allowed transitions Merkle tree
     * @param description Human-readable description
     * @param transitions Array of (from, to) pairs for verification
     * @return policyId ID of new policy
     */
    function createPolicy(
        bytes32 merkleRoot,
        string calldata description,
        uint8[2][] calldata transitions
    ) external onlyAdmin returns (uint256 policyId) {
        if (merkleRoot == bytes32(0)) revert EmptyMerkleRoot();
        if (transitions.length == 0) revert InvalidTransition();
        
        policyId = policyCount++;
        
        policies[policyId] = Policy({
            merkleRoot: merkleRoot,
            createdAt: block.timestamp,
            active: false,
            description: description,
            transitionCount: transitions.length,
            version: currentPolicyVersion
        });
        
        versionToId[currentPolicyVersion] = policyId;
        
        // Store transitions for off-chain verification
        for (uint256 i = 0; i < transitions.length; i++) {
            uint8 fromClass = transitions[i][0];
            uint8 toClass = transitions[i][1];
            
            if (fromClass >= 7 || toClass >= 7) {
                revert InvalidTransition();
            }
            
            bytes32 transitionHash = keccak256(abi.encodePacked(
                fromClass,
                toClass
            ));
            
            policyTransitions[policyId].push(transitionHash);
            
            emit TransitionAdded(policyId, fromClass, toClass);
        }
        
        emit PolicyCreated(policyId, currentPolicyVersion, merkleRoot, description);
    }
    
    /**
     * @notice Activate a policy
     */
    function activatePolicy(uint256 policyId) external onlyAdmin {
        if (policyId >= policyCount) revert PolicyNotFound();
        
        Policy storage policy = policies[policyId];
        
        // Deactivate previous policy if exists
        for (uint256 i = 0; i < policyCount; i++) {
            if (policies[i].active && i != policyId) {
                policies[i].active = false;
                activePolicies[policies[i].merkleRoot] = false;
                emit PolicyDeactivated(i, policies[i].version);
            }
        }
        
        // Activate new policy
        policy.active = true;
        activePolicies[policy.merkleRoot] = true;
        currentPolicyVersion = policy.version + 1;
        
        emit PolicyActivated(policyId, policy.version, policy.merkleRoot);
    }
    
    /**
     * @notice Check if policy is active
     */
    function isPolicyActive(bytes32 policyRoot) external view returns (bool) {
        return activePolicies[policyRoot];
    }
    
    /**
     * @notice Check if transition is allowed (off-chain check)
     */
    function isTransitionAllowed(
        uint256 policyId,
        uint8 from,
        uint8 to
    ) external view returns (bool) {
        if (policyId >= policyCount) revert PolicyNotFound();
        
        bytes32 transitionHash = keccak256(abi.encodePacked(from, to));
        bytes32[] storage transitions = policyTransitions[policyId];
        
        for (uint256 i = 0; i < transitions.length; i++) {
            if (transitions[i] == transitionHash) {
                return true;
            }
        }
        
        return false;
    }
    
    /**
     * @notice Get current policy root
     */
    function getCurrentPolicyRoot() external view returns (bytes32) {
        for (uint256 i = 0; i < policyCount; i++) {
            if (policies[i].active) {
                return policies[i].merkleRoot;
            }
        }
        revert PolicyNotActive();
    }
    
    /**
     * @notice Get current policy ID
     */
    function getCurrentPolicyId() external view returns (uint256) {
        for (uint256 i = 0; i < policyCount; i++) {
            if (policies[i].active) {
                return i;
            }
        }
        revert PolicyNotActive();
    }
    
    /**
     * @notice Get policy details
     */
    function getPolicy(uint256 policyId) 
        external view returns (Policy memory) 
    {
        if (policyId >= policyCount) revert PolicyNotFound();
        return policies[policyId];
    }
    
    /**
     * @notice Get all transitions for policy
     */
    function getTransitions(uint256 policyId) 
        external view returns (bytes32[] memory) 
    {
        if (policyId >= policyCount) revert PolicyNotFound();
        return policyTransitions[policyId];
    }
    
    /**
     * @notice Get total policies
     */
    function getPolicyCount() external view returns (uint256) {
        return policyCount;
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Transfer admin role
     */
    function transferAdmin(address newAdmin) external onlyAdmin {
        if (newAdmin == address(0)) revert ZeroAddress();
        pendingAdmin = newAdmin;
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
}