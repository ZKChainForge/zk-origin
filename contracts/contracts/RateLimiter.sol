// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract RateLimiter is ReentrancyGuard {
    
    uint8 public constant ORIGIN_GENESIS = 0;
    uint8 public constant ORIGIN_USER = 1;
    uint8 public constant ORIGIN_ADMIN = 2;
    uint8 public constant ORIGIN_BRIDGE = 3;
    uint8 public constant ORIGIN_GOVERNANCE = 4;
    uint8 public constant ORIGIN_SYSTEM = 5;
    uint8 public constant ORIGIN_EMERGENCY = 6;
    
    uint256 public constant EPOCH_DURATION = 86400;
    uint256 public constant NUM_ORIGIN_CLASSES = 7;
    
    uint256 public immutable genesisTime;
    
    address public admin;
    address public pendingAdmin;
    address public lineageVerifier;
    
    uint32[7] public rateLimits;
    
    mapping(uint256 => mapping(uint8 => uint32)) public epochCounters;
    mapping(uint256 => bytes32) public epochCounterCommitments;
    mapping(uint256 => bool) public epochCountersReset;
    
    bool public paused;
    
    event RateLimitUpdated(uint8 indexed originClass, uint32 newLimit);
    event CounterIncremented(uint256 indexed epoch, uint8 indexed originClass, uint32 newCount);
    event CounterCommitmentStored(uint256 indexed epoch, bytes32 commitment);
    event CountersResetForEpoch(uint256 indexed epoch);
    event LineageVerifierUpdated(address indexed newVerifier);
    event AdminTransferred(address indexed newAdmin);
    event PausedStateChanged(bool isPausedNow);
    
    error NotAdmin();
    error NotAuthorized();
    error NotPendingAdmin();
    error InvalidOriginClass();
    error RateLimitExceeded(uint8 originClass);
    error CounterCommitmentMismatch();
    error ZeroAddress();
    error InvalidEpoch(uint256 provided, uint256 current);
    error ContractPaused();
    error CounterOverflow();
    error InvalidGenesisTime();
    
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }
    
    modifier onlyAuthorized() {
        if (msg.sender != admin && msg.sender != lineageVerifier) {
            revert NotAuthorized();
        }
        _;
    }
    
    modifier whenNotPaused() {
        if (paused) revert ContractPaused();
        _;
    }
    
    constructor(uint256 _genesisTime) {
        if (_genesisTime == 0) revert InvalidGenesisTime();
        if (_genesisTime > block.timestamp) revert InvalidGenesisTime();
        
        admin = msg.sender;
        genesisTime = _genesisTime;
        paused = false;
        
        rateLimits[ORIGIN_GENESIS] = 1;
        rateLimits[ORIGIN_USER] = type(uint32).max;
        rateLimits[ORIGIN_ADMIN] = 10;
        rateLimits[ORIGIN_BRIDGE] = 100;
        rateLimits[ORIGIN_GOVERNANCE] = 5;
        rateLimits[ORIGIN_SYSTEM] = 1000;
        rateLimits[ORIGIN_EMERGENCY] = 1;
    }
    
    function getCurrentEpoch() public view returns (uint256) {
        if (block.timestamp < genesisTime) revert InvalidGenesisTime();
        return (block.timestamp - genesisTime) / EPOCH_DURATION;
    }
    
    function canProceed(uint256 epoch, uint8 originClass) external view returns (bool) {
        if (originClass >= NUM_ORIGIN_CLASSES) return false;
        
        uint32 current = epochCounters[epoch][originClass];
        uint32 limit = rateLimits[originClass];
        
        if (limit == type(uint32).max) return true;
        return current < limit;
    }
    
    // FIXED: No nonReentrant (no external calls)
    function incrementCounter(uint256 epoch, uint8 originClass) 
        external 
        onlyAuthorized 
        whenNotPaused 
    {
        if (originClass >= NUM_ORIGIN_CLASSES) revert InvalidOriginClass();
        
        uint32 current = epochCounters[epoch][originClass];
        uint32 limit = rateLimits[originClass];
        
        if (limit != type(uint32).max && current >= limit) {
            revert RateLimitExceeded(originClass);
        }
        
        if (current == type(uint32).max) revert CounterOverflow();
        
        epochCounters[epoch][originClass] = current + 1;
        emit CounterIncremented(epoch, originClass, current + 1);
    }
    
    function storeCounterCommitment(uint256 epoch, bytes32 commitment) 
        external 
        onlyAuthorized 
        whenNotPaused 
    {
        bytes32 existing = epochCounterCommitments[epoch];
        
        if (existing != bytes32(0) && existing != commitment) {
            revert CounterCommitmentMismatch();
        }
        
        epochCounterCommitments[epoch] = commitment;
        emit CounterCommitmentStored(epoch, commitment);
    }
    
    // FIXED: Epoch bounds and commitment clearing
    function resetCountersForEpoch(uint256 epoch) 
        external 
        onlyAuthorized 
        whenNotPaused 
    {
        uint256 currentEpoch = getCurrentEpoch();
        
        // FIXED: Allow current epoch or any past epoch (for missed transitions)
        if (epoch > currentEpoch + 1) {
            revert InvalidEpoch(epoch, currentEpoch);
        }
        
        for (uint8 i = 0; i < NUM_ORIGIN_CLASSES; i++) {
            epochCounters[epoch][i] = 0;
        }
        
        // FIXED: Clear commitment on reset
        epochCounterCommitments[epoch] = bytes32(0);
        epochCountersReset[epoch] = true;
        
        emit CountersResetForEpoch(epoch);
    }
    
    function updateRateLimit(uint8 originClass, uint32 newLimit)
        external 
        onlyAdmin 
        whenNotPaused 
    {
        if (originClass >= NUM_ORIGIN_CLASSES) revert InvalidOriginClass();
        rateLimits[originClass] = newLimit;
        emit RateLimitUpdated(originClass, newLimit);
    }
    
    function getCounter(uint256 epoch, uint8 originClass)
        external 
        view 
        returns (uint32)
    {
        if (originClass >= NUM_ORIGIN_CLASSES) return 0;
        return epochCounters[epoch][originClass];
    }
    
    function getLimit(uint8 originClass)
        external 
        view 
        returns (uint32)
    {
        if (originClass >= NUM_ORIGIN_CLASSES) return 0;
        return rateLimits[originClass];
    }
    
    function setLineageVerifier(address _verifier)
        external 
        onlyAdmin 
        whenNotPaused 
    {
        if (_verifier == address(0)) revert ZeroAddress();
        lineageVerifier = _verifier;
        emit LineageVerifierUpdated(_verifier);
    }
    
    function setPaused(bool _paused) external onlyAdmin {
        paused = _paused;
        emit PausedStateChanged(_paused);
    }
    
    function transferAdmin(address newAdmin) external onlyAdmin {
        if (newAdmin == address(0)) revert ZeroAddress();
        pendingAdmin = newAdmin;
    }
    
    function acceptAdmin() external {
        if (msg.sender != pendingAdmin) revert NotPendingAdmin();
        admin = pendingAdmin;
        pendingAdmin = address(0);
        emit AdminTransferred(admin);
    }
}