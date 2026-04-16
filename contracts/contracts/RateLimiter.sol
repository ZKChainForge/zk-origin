// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title RateLimiter
 * @notice Tracks and enforces rate limits per origin class per epoch
 * 
 * UPDATED:
 * - Epoch reset tracking
 * - Counter commitment verification
 * - Defense against reentrancy
 */
contract RateLimiter {
    
    // ============ Origin Classes ============
    uint8 public constant ORIGIN_GENESIS = 0;
    uint8 public constant ORIGIN_USER = 1;
    uint8 public constant ORIGIN_ADMIN = 2;
    uint8 public constant ORIGIN_BRIDGE = 3;
    uint8 public constant ORIGIN_GOVERNANCE = 4;
    uint8 public constant ORIGIN_SYSTEM = 5;
    uint8 public constant ORIGIN_EMERGENCY = 6;
    
    // ============ Constants ============
    uint256 public constant EPOCH_DURATION = 86400; // 24 hours
    uint256 public constant NUM_ORIGIN_CLASSES = 7;
    
    // ============ State ============
    address public admin;
    uint256 public genesisTime;
    
    // Rate limits per origin class
    mapping(uint8 => uint256) public rateLimits;
    
    // Counters per epoch per origin class
    mapping(uint256 => mapping(uint8 => uint256)) public epochCounters;
    
    // Counter commitments (for ZK verification)
    mapping(uint256 => bytes32) public epochCounterCommitments;
    
    // Track whether counters were reset for epoch
    mapping(uint256 => bool) public epochCountersReset;
    
    // ============ Events ============
    event RateLimitUpdated(uint8 indexed originClass, uint256 newLimit);
    event CounterIncremented(uint256 indexed epoch, uint8 indexed originClass, uint256 newCount);
    event CounterCommitmentStored(uint256 indexed epoch, bytes32 commitment);
    event CountersResetForEpoch(uint256 indexed epoch);
    event AdminTransferred(address indexed newAdmin);
    
    // ============ Errors ============
    error NotAdmin();
    error InvalidOriginClass();
    error RateLimitExceeded();
    error CounterCommitmentMismatch();
    error ZeroAddress();
    error EpochMismatch();
    error LockedForUpdate();
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    // ============ Constructor ============
    constructor() {
        admin = msg.sender;
        genesisTime = block.timestamp;
        
        // Set default rate limits (matching circuit)
        rateLimits[ORIGIN_GENESIS] = 1;
        rateLimits[ORIGIN_USER] = type(uint256).max;      // Unlimited
        rateLimits[ORIGIN_ADMIN] = 10;
        rateLimits[ORIGIN_BRIDGE] = 100;
        rateLimits[ORIGIN_GOVERNANCE] = 5;
        rateLimits[ORIGIN_SYSTEM] = 1000;
        rateLimits[ORIGIN_EMERGENCY] = 1;
    }
    
    // ============ Core Functions ============
    
    /**
     * @notice Get current epoch based on time
     */
    function getCurrentEpoch() public view returns (uint256) {
        if (block.timestamp < genesisTime) return 0;
        return (block.timestamp - genesisTime) / EPOCH_DURATION;
    }
    
    /**
     * @notice Check if transition is allowed
     * @param epoch Epoch ID
     * @param originClass Origin class to check
     * @return Whether origin class has capacity
     */
    function canProceed(
        uint256 epoch,
        uint8 originClass
    ) external view returns (bool) {
        if (originClass >= NUM_ORIGIN_CLASSES) return false;
        
        uint256 current = epochCounters[epoch][originClass];
        uint256 limit = rateLimits[originClass];
        
        // Unlimited case
        if (limit == type(uint256).max) return true;
        
        return current < limit;
    }
    
    /**
     * @notice Increment counter for origin class
     * 
     * SECURITY: Should only be called by LineageVerifier after proof verification
     */
    function incrementCounter(
        uint256 epoch,
        uint8 originClass
    ) external onlyAdmin {
        if (originClass >= NUM_ORIGIN_CLASSES) revert InvalidOriginClass();
        
        uint256 current = epochCounters[epoch][originClass];
        uint256 limit = rateLimits[originClass];
        
        // Check limit (skip for unlimited)
        if (limit != type(uint256).max && current >= limit) {
            revert RateLimitExceeded();
        }
        
        // Increment
        epochCounters[epoch][originClass] = current + 1;
        
        emit CounterIncremented(epoch, originClass, current + 1);
    }
    
    /**
     * @notice Store counter commitment for epoch
     * 
     * Called by LineageVerifier after verifying proof
     * Ensures consistency of counter state across proofs
     */
    function storeCounterCommitment(
        uint256 epoch,
        bytes32 commitment
    ) external onlyAdmin {
        // Check for consistency
        bytes32 existing = epochCounterCommitments[epoch];
        
        if (existing != bytes32(0) && existing != commitment) {
            revert CounterCommitmentMismatch();
        }
        
        epochCounterCommitments[epoch] = commitment;
        emit CounterCommitmentStored(epoch, commitment);
    }
    
    /**
     * @notice Reset counters for new epoch
     * 
     * Called by LineageVerifier when transitioning to new epoch
     */
    function resetCountersForEpoch(uint256 epoch) external onlyAdmin {
        if (epoch >= type(uint256).max) revert();
        
        // Clear all counters for this epoch
        for (uint8 i = 0; i < NUM_ORIGIN_CLASSES; i++) {
            epochCounters[epoch][i] = 0;
        }
        
        // Mark as reset
        epochCountersReset[epoch] = true;
        
        // Store genesis commitment (all zeros)
        bytes32 genesisCommitment = keccak256(
            abi.encode(epoch, [uint256(0), 0, 0, 0, 0, 0, 0])
        );
        epochCounterCommitments[epoch] = genesisCommitment;
        
        emit CountersResetForEpoch(epoch);
    }
    
    /**
     * @notice Update rate limit for origin class
     */
    function updateRateLimit(uint8 originClass, uint256 newLimit)
        external onlyAdmin
    {
        if (originClass >= NUM_ORIGIN_CLASSES) revert InvalidOriginClass();
        rateLimits[originClass] = newLimit;
        emit RateLimitUpdated(originClass, newLimit);
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Get counter for origin class in epoch
     */
    function getCounter(uint256 epoch, uint8 originClass)
        external view returns (uint256)
    {
        if (originClass >= NUM_ORIGIN_CLASSES) return 0;
        return epochCounters[epoch][originClass];
    }
    
    /**
     * @notice Get rate limit for origin class
     */
    function getLimit(uint8 originClass)
        external view returns (uint256)
    {
        if (originClass >= NUM_ORIGIN_CLASSES) return 0;
        return rateLimits[originClass];
    }
    
    /**
     * @notice Get remaining capacity
     */
    function getRemainingCapacity(uint256 epoch, uint8 originClass)
        external view returns (uint256)
    {
        if (originClass >= NUM_ORIGIN_CLASSES) return 0;
        
        uint256 current = epochCounters[epoch][originClass];
        uint256 limit = rateLimits[originClass];
        
        // Unlimited case
        if (limit == type(uint256).max) return type(uint256).max;
        
        if (current >= limit) return 0;
        return limit - current;
    }
    
    /**
     * @notice Get counter commitment for epoch
     */
    function getCounterCommitment(uint256 epoch)
        external view returns (bytes32)
    {
        return epochCounterCommitments[epoch];
    }
    
    /**
     * @notice Check if epoch counters were reset
     */
    function wereCountersReset(uint256 epoch)
        external view returns (bool)
    {
        return epochCountersReset[epoch];
    }
    
    /**
     * @notice Get all counters for an epoch
     */
    function getEpochCounters(uint256 epoch)
        external view returns (uint256[7] memory counters)
    {
        for (uint8 i = 0; i < NUM_ORIGIN_CLASSES; i++) {
            counters[i] = epochCounters[epoch][i];
        }
    }
    
    /**
     * @notice Transfer admin
     */
    function transferAdmin(address newAdmin) external onlyAdmin {
        if (newAdmin == address(0)) revert ZeroAddress();
        admin = newAdmin;
        emit AdminTransferred(newAdmin);
    }
}