// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title MockGroth16Verifier
 * @notice Mock Groth16 verifier for testing
 * 
 * ALWAYS RETURNS TRUE - ONLY FOR TESTING!
 * DO NOT USE IN PRODUCTION!
 */
contract MockGroth16Verifier {
    
    /**
     * @notice Verify a Groth16 proof (mock implementation)
     * @dev This always returns true for testing purposes
     * @return Always returns true
     */
    function verifyProof(
        uint[2] calldata _pA,
        uint[2][2] calldata _pB,
        uint[2] calldata _pC,
        uint[12] calldata _pubSignals
    ) external pure returns (bool) {
        // Mock implementation: always return true
        return true;
    }
}