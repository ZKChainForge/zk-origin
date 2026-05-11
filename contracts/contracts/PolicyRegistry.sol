// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title PolicyRegistry
 * @notice Manages origin transition policies with versioning and timelock
 * 
 * FIXED issues from previous version:
 * 1. versionToId overwrite: version incremented per policy
 * 2. activePolicies keyed by policyId not merkleRoot
 * 3. proposePolicyActivation cannot reset already-pending timelock
 * 4. createPolicy activation time only set at activation proposal time
 * 5. Transitions stored as (from, to) pairs not just hashes (readable)
 */
contract PolicyRegistry is ReentrancyGuard {

    uint256 public constant POLICY_ACTIVATION_TIMELOCK = 2 days;

    struct Policy {
        bytes32 merkleRoot;
        uint64  createdAt;
        uint64  activatesAt;       // Set by proposePolicyActivation
        uint64  version;
        uint32  transitionCount;
        bool    active;
        bool    proposalPending;   // Has activation been proposed?
    }

    struct TransitionPair {
        uint8 fromClass;
        uint8 toClass;
    }

    // ===== State =====
    address public admin;
    address public pendingAdmin;

    uint256 public policyCount;
    uint256 public currentPolicyVersion;

    // Track active policy by policyId (not merkleRoot)
    uint256 public currentActivePolicyId;
    bool    public hasActivePolicy;

    mapping(uint256 => Policy) public policies;

    // policyId → transitionHash → allowed (O(1) lookup)
    mapping(uint256 => mapping(bytes32 => bool)) public policyTransitionAllowed;

    // policyId → readable transition pairs
    mapping(uint256 => TransitionPair[]) public policyTransitions;

    bool public paused;

    // ===== Events =====
    event PolicyCreated(uint256 indexed policyId, uint256 indexed version, bytes32 merkleRoot);
    event PolicyActivationProposed(uint256 indexed policyId, uint256 activatesAt);
    event PolicyActivated(uint256 indexed policyId, uint256 indexed version, bytes32 merkleRoot);
    event PolicyDeactivated(uint256 indexed policyId);
    event AdminTransferred(address indexed newAdmin);
    event PausedStateChanged(bool isPausedNow);

    // ===== Errors =====
    error NotAdmin();
    error NotPendingAdmin();
    error PolicyNotFound();
    error PolicyNotActive();
    error InvalidTransition();
    error ZeroAddress();
    error EmptyMerkleRoot();
    error ZeroTransitionCount();
    error TooManyTransitions();
    error DuplicateTransition();
    error TimelockNotExpired(uint256 current, uint256 activatesAt);
    error ContractPaused();
    error ProposalAlreadyPending();
    error NoProposalPending();
    error AlreadyActivePolicy();
    error InvalidPolicyVersion();

    // ===== Modifiers =====
    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }

    modifier whenNotPaused() {
        if (paused) revert ContractPaused();
        _;
    }

    // ===== Constructor =====
    constructor() {
        admin = msg.sender;
        currentPolicyVersion = 1;
        policyCount = 0;
        hasActivePolicy = false;
        currentActivePolicyId = type(uint256).max;
        paused = false;
    }

    // ===== createPolicy =====

    /**
     * @notice Create a new policy
     * @param merkleRoot  Merkle root of allowed transitions
     * @param transitions Array of [fromClass, toClass] pairs
     * @return policyId   ID of created policy
     * 
     * FIXED: version increments per policy (no overwrite)
     * FIXED: activation time NOT set here (set at proposal time)
     */
    function createPolicy(
        bytes32 merkleRoot,
        uint8[2][] calldata transitions
    )
        external
        onlyAdmin
        whenNotPaused
        nonReentrant
        returns (uint256 policyId)
    {
        if (merkleRoot == bytes32(0)) revert EmptyMerkleRoot();
        if (transitions.length == 0) revert ZeroTransitionCount();
        if (transitions.length > 49) revert TooManyTransitions(); // max 7x7

        policyId = policyCount;
        policyCount++;

        policies[policyId] = Policy({
            merkleRoot:      merkleRoot,
            createdAt:       uint64(block.timestamp),
            activatesAt:     0,         // Set when proposed
            version:         uint64(currentPolicyVersion),
            transitionCount: uint32(transitions.length),
            active:          false,
            proposalPending: false
        });

        // FIXED: increment version per policy (no overwrite)
        currentPolicyVersion++;

        // Store transitions
        for (uint256 i = 0; i < transitions.length; i++) {
            uint8 fromClass = transitions[i][0];
            uint8 toClass   = transitions[i][1];

            if (fromClass >= 7 || toClass >= 7) revert InvalidTransition();

            bytes32 transitionHash = keccak256(abi.encodePacked(fromClass, toClass));
            if (policyTransitionAllowed[policyId][transitionHash]) {
                revert DuplicateTransition();
            }

            policyTransitionAllowed[policyId][transitionHash] = true;
            policyTransitions[policyId].push(TransitionPair(fromClass, toClass));
        }

        emit PolicyCreated(policyId, policies[policyId].version, merkleRoot);
    }

    // ===== proposePolicyActivation =====

    /**
     * @notice Propose activation (starts 2-day timelock)
     * 
     * FIXED: Cannot re-propose if already pending (prevents timelock reset attack)
     * FIXED: Activation time set HERE not in createPolicy
     */
    function proposePolicyActivation(uint256 policyId)
        external
        onlyAdmin
        whenNotPaused
    {
        if (policyId >= policyCount) revert PolicyNotFound();

        Policy storage policy = policies[policyId];

        // Prevent re-proposing (would reset timelock)
        if (policy.proposalPending) revert ProposalAlreadyPending();
        if (policy.active) revert AlreadyActivePolicy();

        policy.activatesAt     = uint64(block.timestamp + POLICY_ACTIVATION_TIMELOCK);
        policy.proposalPending = true;

        emit PolicyActivationProposed(policyId, policy.activatesAt);
    }

    // ===== activatePolicy =====

    /**
     * @notice Activate policy after timelock expires
     * 
     * FIXED: Keyed by policyId not merkleRoot
     * FIXED: O(1) deactivation of previous
     * FIXED: Requires proposal to have been made (no skip)
     */
    function activatePolicy(uint256 policyId)
        external
        onlyAdmin
        whenNotPaused
        nonReentrant
    {
        if (policyId >= policyCount) revert PolicyNotFound();

        Policy storage newPolicy = policies[policyId];

        // Must have been proposed
        if (!newPolicy.proposalPending) revert NoProposalPending();
        if (newPolicy.active) revert AlreadyActivePolicy();

        // Timelock must have expired
        if (block.timestamp < newPolicy.activatesAt) {
            revert TimelockNotExpired(block.timestamp, newPolicy.activatesAt);
        }

        // Deactivate current policy (O(1))
        if (hasActivePolicy) {
            Policy storage oldPolicy = policies[currentActivePolicyId];
            oldPolicy.active = false;
            emit PolicyDeactivated(currentActivePolicyId);
        }

        // Activate new policy
        newPolicy.active          = true;
        newPolicy.proposalPending = false;
        currentActivePolicyId     = policyId;
        hasActivePolicy           = true;

        emit PolicyActivated(policyId, newPolicy.version, newPolicy.merkleRoot);
    }

    // ===== View Functions =====

    function getCurrentPolicyRoot() external view returns (bytes32) {
        if (!hasActivePolicy) revert PolicyNotActive();
        return policies[currentActivePolicyId].merkleRoot;
    }

    function getCurrentPolicyId() external view returns (uint256) {
        if (!hasActivePolicy) revert PolicyNotActive();
        return currentActivePolicyId;
    }

    /**
     * @notice Check if transition allowed in current active policy
     */
    function isTransitionAllowed(uint8 from, uint8 to)
        external view returns (bool)
    {
        if (!hasActivePolicy) return false;
        if (from >= 7 || to >= 7) return false;
        bytes32 h = keccak256(abi.encodePacked(from, to));
        return policyTransitionAllowed[currentActivePolicyId][h];
    }

    /**
     * @notice Get readable transitions for a policy
     * FIXED: Returns actual (from, to) pairs not opaque hashes
     */
    function getTransitions(uint256 policyId)
        external view returns (TransitionPair[] memory)
    {
        if (policyId >= policyCount) revert PolicyNotFound();
        return policyTransitions[policyId];
    }

    function getPolicy(uint256 policyId)
        external view returns (Policy memory)
    {
        if (policyId >= policyCount) revert PolicyNotFound();
        return policies[policyId];
    }

    function getTimeUntilActivation(uint256 policyId)
        external view returns (uint256)
    {
        if (policyId >= policyCount) revert PolicyNotFound();
        uint64 at = policies[policyId].activatesAt;
        if (at == 0 || block.timestamp >= at) return 0;
        return at - block.timestamp;
    }

    function getPolicyCount() external view returns (uint256) {
        return policyCount;
    }

    // ===== Admin =====

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