// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IAuthorizationVerifier
 * @notice Interface for authorization proof verification
 */
interface IAuthorizationVerifier {
    
    enum AuthType {
        User,        // 0: Single signature
        Admin,       // 1: M-of-N multisig
        Bridge,      // 2: Bridge attestation
        Governance,  // 3: Governance vote
        System,      // 4: System call
        Emergency    // 5: Emergency key
    }
    
    /**
     * @notice Verify authorization
     * @param authType Type of authorization
     * @param data Authorization data (signatures, commitments, etc)
     * @return valid Whether authorization is valid
     */
    function verifyAuthorization(
        AuthType authType,
        bytes calldata data
    ) external view returns (bool valid);
    
    /**
     * @notice Get authorization commitment
     * @param authType Type of authorization
     * @param data Authorization data
     * @return commitment Commitment to authorization
     */
    function getAuthorizationCommitment(
        AuthType authType,
        bytes calldata data
    ) external view returns (bytes32 commitment);
}