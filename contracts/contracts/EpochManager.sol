// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title EpochManager
 * @notice Manages epochs for state reset and counter management
 */
contract EpochManager {
    
    uint256 public constant EPOCH_DURATION = 86400; // 24 hours
    
    uint256 public genesisTime;
    uint256 public currentEpoch;
    address public admin;
    
    mapping(uint256 => bool) public epochCountersReset;
    
    event EpochChanged(uint256 indexed epoch, uint256 timestamp);
    
    error NotAdmin();
    error NoTimeTravel();
    
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    constructor() {
        admin = msg.sender;
        genesisTime = block.timestamp;
        currentEpoch = 0;
    }
    
    /**
     * @notice Get current epoch
     */
    function getCurrentEpoch() external view returns (uint256) {
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
     * @notice Update current epoch
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
    }
}