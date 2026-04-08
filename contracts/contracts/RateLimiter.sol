// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title RateLimiter
 * @notice Tracks and enforces rate limits per origin class per epoch
 */
contract RateLimiter {
    
    // Origin classes
    uint8 public constant ORIGIN_GENESIS = 0;
    uint8 public constant ORIGIN_USER = 1;
    uint8 public constant ORIGIN_ADMIN = 2;
    uint8 public constant ORIGIN_BRIDGE = 3;
    uint8 public constant ORIGIN_GOVERNANCE = 4;
    uint8 public constant ORIGIN_SYSTEM = 5;
    uint8 public constant ORIGIN_EMERGENCY = 6;
    
    // ============ State ============
    address public admin;
    
    // Rate limits per origin class per epoch
    mapping(uint256 => mapping(uint8 => uint256)) public epochCounters;
    
    // Max limits for each origin class
    mapping(uint8 => uint256) public rateLimits;
    
    // ============ Events ============
    event RateLimitUpdated(uint8 indexed originClass, uint256 newLimit);
    event CounterIncremented(uint256 indexed epoch, uint8 indexed originClass, uint256 newCount);
    event CounterReset(uint256 indexed epoch, uint8 indexed originClass);
    
    // ============ Errors ============
    error NotAdmin();
    error InvalidOriginClass();
    error RateLimitExceeded();
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    // ============ Constructor ============
    constructor() {
        admin = msg.sender;
        
        // Set default rate limits (matching circuit)
        rateLimits[ORIGIN_GENESIS] = 1;
        rateLimits[ORIGIN_USER] = type(uint256).max;      // Unlimited
        rateLimits[ORIGIN_ADMIN] = 10;
        rateLimits[ORIGIN_BRIDGE] = 100;
        rateLimits[ORIGIN_GOVERNANCE] = 5;
        rateLimits[ORIGIN_SYSTEM] = 1000;
        rateLimits[ORIGIN_EMERGENCY] = 1;
    }
    
    // ============ Rate Limit Management ============
    
    /**
     * @notice Check if origin class can proceed
     * @param epoch Current epoch
     * @param originClass Origin class to check
     * @return Whether origin class has capacity
     */
    function canProceed(
        uint256 epoch,
        uint8 originClass
    ) external view returns (bool) {
        if (originClass > 6) return false;
        
        uint256 current = epochCounters[epoch][originClass];
        uint256 limit = rateLimits[originClass];
        
        return current < limit;
    }
    
    /**
     * @notice Increment counter for origin class
     * @param epoch Current epoch
     * @param originClass Origin class to increment
     */
    function incrementCounter(
        uint256 epoch,
        uint8 originClass
    ) external onlyAdmin {
        if (originClass > 6) revert InvalidOriginClass();
        
        uint256 current = epochCounters[epoch][originClass];
        uint256 limit = rateLimits[originClass];
        
        if (current >= limit) revert RateLimitExceeded();
        
        epochCounters[epoch][originClass] = current + 1;
        emit CounterIncremented(epoch, originClass, current + 1);
    }
    
    /**
     * @notice Reset counter for epoch and origin class
     * @param epoch Epoch to reset
     * @param originClass Origin class to reset
     */
    function resetCounter(
        uint256 epoch,
        uint8 originClass
    ) external onlyAdmin {
        if (originClass > 6) revert InvalidOriginClass();
        
        epochCounters[epoch][originClass] = 0;
        emit CounterReset(epoch, originClass);
    }
    
    /**
     * @notice Update rate limit for origin class
     * @param originClass Origin class to update
     * @param newLimit New rate limit
     */
    function updateRateLimit(uint8 originClass, uint256 newLimit)
        external
        onlyAdmin
    {
        if (originClass > 6) revert InvalidOriginClass();
        rateLimits[originClass] = newLimit;
        emit RateLimitUpdated(originClass, newLimit);
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Get counter for origin class in epoch
     */
    function getCounter(uint256 epoch, uint8 originClass)
        external
        view
        returns (uint256)
    {
        if (originClass > 6) return 0;
        return epochCounters[epoch][originClass];
    }
    
    /**
     * @notice Get rate limit for origin class
     */
    function getLimit(uint8 originClass)
        external
        view
        returns (uint256)
    {
        if (originClass > 6) return 0;
        return rateLimits[originClass];
    }
    
    /**
     * @notice Get remaining capacity for origin class
     */
    function getRemainingCapacity(uint256 epoch, uint8 originClass)
        external
        view
        returns (uint256)
    {
        if (originClass > 6) return 0;
        
        uint256 current = epochCounters[epoch][originClass];
        uint256 limit = rateLimits[originClass];
        
        if (current >= limit) return 0;
        return limit - current;
    }
    
    /**
     * @notice Transfer admin
     */
    function transferAdmin(address newAdmin) external onlyAdmin {
        admin = newAdmin;
    }
}