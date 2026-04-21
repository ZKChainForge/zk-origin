// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title EpochManager (PRODUCTION)
 * @notice Manages epochs for state reset and counter management
 * 
 * SECURITY:
 *  Epoch duration fixed (24 hours)
 *  Epoch cannot go backwards
 *  Monotonic increasing only
 *  Timestamp validation
 * 
 * EPOCHS:
 * - Duration: 24 hours = 86400 seconds
 * - Used for rate limit resets
 * - Prevents counter overflow
 * - Enables temporal security properties
 */

contract EpochManager {
    
    // ============ Constants ============
    uint256 public constant EPOCH_DURATION = 86400;  // 24 hours
    
    // ============ State ============
    uint256 public genesisTime;
    uint256 public currentEpoch;
    address public admin;
    
    // Track when counters were reset for each epoch
    mapping(uint256 => bool) public epochCountersReset;
    
    // ============ Events ============
    event EpochChanged(uint256 indexed epoch, uint256 timestamp);
    event CountersResetForEpoch(uint256 indexed epoch);
    event AdminTransferred(address indexed newAdmin);
    
    // ============ Errors ============
    error NotAdmin();
    error NoTimeTravel();
    error ZeroAddress();
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    // ============ Constructor ============
    constructor() {
        admin = msg.sender;
        genesisTime = block.timestamp;
        currentEpoch = 0;
    }
    
    // ============ Core Functions ============
    
    /**
     * @notice Get current epoch based on block timestamp
     */
    function getCurrentEpoch() external view returns (uint256) {
        if (block.timestamp < genesisTime) revert NoTimeTravel();
        return (block.timestamp - genesisTime) / EPOCH_DURATION;
    }
    
    /**
     * @notice Check if epoch has changed
     */
    function hasEpochChanged() external view returns (bool) {
        uint256 newEpoch = (block.timestamp - genesisTime) / EPOCH_DURATION;
        return newEpoch > currentEpoch;
    }
    
    /**
     * @notice Get time until next epoch
     */
    function timeUntilNextEpoch() external view returns (uint256) {
        uint256 nextEpochTime = genesisTime + ((currentEpoch + 1) * EPOCH_DURATION);
        if (block.timestamp >= nextEpochTime) return 0;
        return nextEpochTime - block.timestamp;
    }
    
    /**
     * @notice Update current epoch if needed
     */
    function updateEpoch() external {
        uint256 newEpoch = (block.timestamp - genesisTime) / EPOCH_DURATION;
        if (newEpoch > currentEpoch) {
            currentEpoch = newEpoch;
            emit EpochChanged(newEpoch, block.timestamp);
        }
    }
    
    /**
     * @notice Mark epoch counters as reset
     */
    function markCountersReset(uint256 epoch) external onlyAdmin {
        epochCountersReset[epoch] = true;
        emit CountersResetForEpoch(epoch);
    }
    
    /**
     * @notice Check if epoch counters were reset
     */
    function wereCountersReset(uint256 epoch) external view returns (bool) {
        return epochCountersReset[epoch];
    }
    
    /**
     * @notice Get epoch duration
     */
    function getEpochDuration() external pure returns (uint256) {
        return EPOCH_DURATION;
    }
    
    /**
     * @notice Get genesis time
     */
    function getGenesisTime() external view returns (uint256) {
        return genesisTime;
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