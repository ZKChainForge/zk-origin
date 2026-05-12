
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// Minimal mock for local testing without full V4 core
contract MockPoolManager {

    event PoolInitialized(bytes32 indexed poolId);
    event SwapExecuted(bytes32 indexed poolId, address sender);
    event DonationExecuted(bytes32 indexed poolId, uint256 amount0, uint256 amount1);
    event LiquidityAdded(bytes32 indexed poolId, address sender);
    event LiquidityRemoved(bytes32 indexed poolId, address sender);

    mapping(bytes32 => bool) public initializedPools;

    function initializePool(bytes32 poolId) external {
        initializedPools[poolId] = true;
        emit PoolInitialized(poolId);
    }

    function isPoolInitialized(bytes32 poolId) external view returns (bool) {
        return initializedPools[poolId];
    }
}

// Mock donation verifier - always returns true for testing
contract MockDonationVerifier {
    function verifyProof(
        uint256[2] calldata,
        uint256[2][2] calldata,
        uint256[2] calldata,
        uint256[12] calldata pubSignals
    ) external pure returns (bool) {
        // In production: actual Groth16 verification
        // For testing: check basic signal validity
        require(pubSignals[0] != 0, "MockDonationVerifier: invalid pool ID");
        require(pubSignals[1] > 0, "MockDonationVerifier: zero donation amount");
        return true;
    }
}

// Mock permission verifier - validates basic signal structure
contract MockPermissionVerifier {
    function verifyProof(
        uint256[2] calldata,
        uint256[2][2] calldata,
        uint256[2] calldata,
        uint256[8] calldata pubSignals
    ) external pure returns (bool) {
        // In production: actual Groth16 verification
        // For testing: check basic signal validity
        require(pubSignals[0] != 0, "MockPermissionVerifier: invalid caller state");
        require(pubSignals[2] <= 3, "MockPermissionVerifier: invalid action type");
        require(pubSignals[3] >= 1 && pubSignals[3] <= 6, "MockPermissionVerifier: invalid origin class");
        return true;
    }
}