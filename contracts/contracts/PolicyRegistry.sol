// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title PolicyRegistry
 * @notice Manages origin transition policies with Merkle tree
 */
contract PolicyRegistry {
    
    struct Policy {
        bytes32 merkleRoot;
        uint256 createdAt;
        bool active;
        string description;
        uint256 transitionCount;
    }
    
    address public admin;
    uint256 public currentPolicyId;
    mapping(uint256 => Policy) public policies;
    uint256 public policyCount;
    
    // Transition storage for verification
    mapping(uint256 => mapping(bytes32 => bool)) public policyTransitions;
    
    event PolicyCreated(uint256 indexed policyId, bytes32 merkleRoot, string description);
    event PolicyActivated(uint256 indexed policyId);
    event TransitionAdded(uint256 indexed policyId, uint8 from, uint8 to);
    
    error NotAdmin();
    error PolicyNotFound();
    error PolicyNotActive();
    
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    constructor() {
        admin = msg.sender;
    }
    
    /**
     * @notice Create policy with pre-computed Merkle root
     * @param merkleRoot Root of allowed transitions Merkle tree
     * @param description Human-readable description
     * @param transitions Array of (from, to) pairs for verification
     */
    function createPolicy(
        bytes32 merkleRoot,
        string calldata description,
        uint8[2][] calldata transitions
    ) external onlyAdmin returns (uint256 policyId) {
        policyId = policyCount++;
        
        policies[policyId] = Policy({
            merkleRoot: merkleRoot,
            createdAt: block.timestamp,
            active: false,
            description: description,
            transitionCount: transitions.length
        });
        
        // Store transitions for off-chain verification
        for (uint256 i = 0; i < transitions.length; i++) {
            bytes32 transitionHash = keccak256(abi.encodePacked(
                transitions[i][0],
                transitions[i][1]
            ));
            policyTransitions[policyId][transitionHash] = true;
            
            emit TransitionAdded(policyId, transitions[i][0], transitions[i][1]);
        }
        
        emit PolicyCreated(policyId, merkleRoot, description);
    }
    
    /**
     * @notice Activate a policy
     */
    function activatePolicy(uint256 policyId) external onlyAdmin {
        if (policyId >= policyCount) revert PolicyNotFound();
        
        if (currentPolicyId < policyCount) {
            policies[currentPolicyId].active = false;
        }
        
        policies[policyId].active = true;
        currentPolicyId = policyId;
        
        emit PolicyActivated(policyId);
    }
    
    /**
     * @notice Check if transition is allowed (off-chain check)
     */
    function isTransitionAllowed(uint8 from, uint8 to) external view returns (bool) {
        bytes32 transitionHash = keccak256(abi.encodePacked(from, to));
        return policyTransitions[currentPolicyId][transitionHash];
    }
    
    /**
     * @notice Get current policy root
     */
    function getCurrentPolicyRoot() external view returns (bytes32) {
        if (!policies[currentPolicyId].active) revert PolicyNotActive();
        return policies[currentPolicyId].merkleRoot;
    }
    
    /**
     * @notice Get policy details
     */
    function getPolicy(uint256 policyId) external view returns (Policy memory) {
        if (policyId >= policyCount) revert PolicyNotFound();
        return policies[policyId];
    }
}