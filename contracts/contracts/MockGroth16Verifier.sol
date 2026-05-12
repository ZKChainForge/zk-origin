// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @dev Mock Groth16 verifier that always returns true
 * USE ONLY FOR TESTING - DO NOT DEPLOY TO PRODUCTION
 */
contract MockGroth16Verifier {
    function verifyProof(
        uint[2] memory a,
        uint[2][2] memory b,
        uint[2] memory c,
        uint[20] memory input
    ) public pure returns (bool) {
        // Suppress unused variable warnings
        a; b; c; input;
        
        // ALWAYS RETURN TRUE - for testing only
        return true;
    }
}