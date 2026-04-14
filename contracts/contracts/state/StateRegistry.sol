// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title StateRegistry
 * @notice Stores and manages verified states with full lineage history
 */
contract StateRegistry {
    
    // ============ State Storage ============
    
    struct StateRecord {
        bytes32 stateHash;
        bytes32 lineageCommitment;
        uint256 depth;
        uint8 originClass;
        uint64 timestamp;
        address creator;
        bytes32 prevStateHash;
        bool verified;
    }
    
    struct StateMetadata {
        uint256 createdBlock;
        uint256 createdTime;
        string description;
        bytes customData;
    }
    
    // Main storage
    mapping(bytes32 => StateRecord) public states;
    mapping(bytes32 => StateMetadata) public metadata;
    
    // Indexed access
    bytes32[] public stateHashes;
    mapping(uint256 => bytes32) public stateByIndex;
    
    // Statistics
    uint256 public totalStates;
    uint256 public maxDepthReached;
    bytes32 public genesisState;
    
    // ============ Events ============
    
    event StateRegistered(
        bytes32 indexed stateHash,
        bytes32 indexed prevStateHash,
        uint256 depth,
        uint8 originClass,
        address indexed creator
    );
    
    event StateMetadataUpdated(
        bytes32 indexed stateHash,
        string description
    );
    
    event GenesisSet(
        bytes32 indexed stateHash
    );
    
    // ============ Errors ============
    
    error StateAlreadyRegistered();
    error StateNotFound();
    error InvalidDepth();
    error GenesisAlreadySet();
    
    // ============ Core Functions ============
    
    /**
     * @notice Register a verified state
     */
    function registerState(
        bytes32 stateHash,
        bytes32 lineageCommitment,
        uint256 depth,
        uint8 originClass,
        bytes32 prevStateHash
    ) external {
        require(!states[stateHash].verified, "State already registered");
        require(depth <= 1_000_000, "Depth too large");
        
        // Register state
        states[stateHash] = StateRecord({
            stateHash: stateHash,
            lineageCommitment: lineageCommitment,
            depth: depth,
            originClass: originClass,
            timestamp: uint64(block.timestamp),
            creator: msg.sender,
            prevStateHash: prevStateHash,
            verified: true
        });
        
        // Add to index
        stateHashes.push(stateHash);
        stateByIndex[totalStates] = stateHash;
        
        // Update statistics
        totalStates++;
        if (depth > maxDepthReached) {
            maxDepthReached = depth;
        }
        
        emit StateRegistered(stateHash, prevStateHash, depth, originClass, msg.sender);
    }
    
    /**
     * @notice Set genesis state
     */
    function setGenesis(bytes32 stateHash) external {
        require(genesisState == bytes32(0), "Genesis already set");
        
        genesisState = stateHash;
        
        // Register genesis
        states[stateHash] = StateRecord({
            stateHash: stateHash,
            lineageCommitment: stateHash,
            depth: 0,
            originClass: 0,  // Genesis class
            timestamp: uint64(block.timestamp),
            creator: msg.sender,
            prevStateHash: bytes32(0),
            verified: true
        });
        
        stateHashes.push(stateHash);
        stateByIndex[0] = stateHash;
        totalStates++;
        
        emit GenesisSet(stateHash);
    }
    
    /**
     * @notice Add metadata to a state
     */
    function setMetadata(
        bytes32 stateHash,
        string calldata description,
        bytes calldata customData
    ) external {
        require(states[stateHash].verified, "State not verified");
        
        metadata[stateHash] = StateMetadata({
            createdBlock: block.number,
            createdTime: block.timestamp,
            description: description,
            customData: customData
        });
        
        emit StateMetadataUpdated(stateHash, description);
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Get state record
     */
    function getState(bytes32 stateHash)
        external
        view
        returns (StateRecord memory)
    {
        require(states[stateHash].verified, "State not found");
        return states[stateHash];
    }
    
    /**
     * @notice Get lineage commitment for state
     */
    function getLineageCommitment(bytes32 stateHash)
        external
        view
        returns (bytes32)
    {
        require(states[stateHash].verified, "State not found");
        return states[stateHash].lineageCommitment;
    }
    
    /**
     * @notice Get state depth
     */
    function getDepth(bytes32 stateHash)
        external
        view
        returns (uint256)
    {
        require(states[stateHash].verified, "State not found");
        return states[stateHash].depth;
    }
    
    /**
     * @notice Check if state is verified
     */
    function isVerified(bytes32 stateHash)
        external
        view
        returns (bool)
    {
        return states[stateHash].verified;
    }
    
    /**
     * @notice Get state at index
     */
    function getStateByIndex(uint256 index)
        external
        view
        returns (StateRecord memory)
    {
        require(index < totalStates, "Index out of bounds");
        return states[stateByIndex[index]];
    }
    
    /**
     * @notice Get metadata for state
     */
    function getMetadata(bytes32 stateHash)
        external
        view
        returns (StateMetadata memory)
    {
        return metadata[stateHash];
    }
    
    /**
     * @notice Get all states
     */
    function getAllStates()
        external
        view
        returns (StateRecord[] memory)
    {
        StateRecord[] memory records = new StateRecord[](totalStates);
        
        for (uint256 i = 0; i < totalStates; i++) {
            records[i] = states[stateByIndex[i]];
        }
        
        return records;
    }
    
    /**
     * @notice Get lineage path from genesis to state
     */
    function getLineagePath(bytes32 stateHash)
        external
        view
        returns (bytes32[] memory)
    {
        require(states[stateHash].verified, "State not found");
        
        // Walk backwards from stateHash to genesis
        uint256 depth = states[stateHash].depth;
        bytes32[] memory path = new bytes32[](depth + 1);
        
        bytes32 current = stateHash;
        for (uint256 i = depth; i > 0; i--) {
            path[i] = current;
            current = states[current].prevStateHash;
        }
        path[0] = genesisState;
        
        return path;
    }
}