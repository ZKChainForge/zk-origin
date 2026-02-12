// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title PolicyRegistry
 * @author ZK-ORIGIN Team
 * @notice Manages origin transition policies for ZK-ORIGIN
 * @dev Stores the Merkle root of allowed (from_origin, to_origin) pairs
 */
contract PolicyRegistry {
    
    // Types
    
    struct Policy {
        bytes32 merkleRoot;      // Merkle root of allowed transitions
        uint256 createdAt;       // When the policy was created
        bool active;             // Whether the policy is active
        string description;      // Human-readable description
    }
    
    
    // State Variables
    
    
    /// @notice Admin address
    address public admin;
    
    /// @notice Current active policy ID
    uint256 public currentPolicyId;
    
    /// @notice Mapping from policy ID to policy
    mapping(uint256 => Policy) public policies;
    
    /// @notice Total number of policies
    uint256 public policyCount;
    
    
    // Events
    
    
    event PolicyCreated(uint256 indexed policyId, bytes32 merkleRoot, string description);
    event PolicyActivated(uint256 indexed policyId);
    event PolicyDeactivated(uint256 indexed policyId);
    
    
    // Errors
    
    
    error NotAdmin();
    error PolicyNotFound();
    error PolicyNotActive();
    
    // Constructor
    
    constructor() {
        admin = msg.sender;
    }
    
    // Admin Functions
    
    /**
     * @notice Create a new policy
     * @param merkleRoot Merkle root of allowed transitions
     * @param description Human-readable description
     * @return policyId The new policy ID
     */
    function createPolicy(
        bytes32 merkleRoot,
        string calldata description
    ) external returns (uint256 policyId) {
        if (msg.sender != admin) revert NotAdmin();
        
        policyId = policyCount++;
        policies[policyId] = Policy({
            merkleRoot: merkleRoot,
            createdAt: block.timestamp,
            active: false,
            description: description
        });
        
        emit PolicyCreated(policyId, merkleRoot, description);
    }
    
    /**
     * @notice Activate a policy
     * @param policyId ID of the policy to activate
     */
    function activatePolicy(uint256 policyId) external {
        if (msg.sender != admin) revert NotAdmin();
        if (policyId >= policyCount) revert PolicyNotFound();
        
        // Deactivate current policy
        if (currentPolicyId < policyCount) {
            policies[currentPolicyId].active = false;
            emit PolicyDeactivated(currentPolicyId);
        }
        
        // Activate new policy
        policies[policyId].active = true;
        currentPolicyId = policyId;
        
        emit PolicyActivated(policyId);
    }
    
    /**
     * @notice Transfer admin role
     * @param newAdmin Address of the new admin
     */
    function transferAdmin(address newAdmin) external {
        if (msg.sender != admin) revert NotAdmin();
        admin = newAdmin;
    }
    
    // View Functions
 
    
    /**
     * @notice Get the current policy Merkle root
     * @return merkleRoot The current policy Merkle root
     */
    function getCurrentPolicyRoot() external view returns (bytes32) {
        if (!policies[currentPolicyId].active) revert PolicyNotActive();
        return policies[currentPolicyId].merkleRoot;
    }
    
    /**
     * @notice Get policy details
     * @param policyId ID of the policy
     * @return policy The policy details
     */
    function getPolicy(uint256 policyId) external view returns (Policy memory) {
        if (policyId >= policyCount) revert PolicyNotFound();
        return policies[policyId];
    }
}