// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IAuthorizationVerifier
 * @notice Interface for authorization proof verification
 * 
 * UPDATED: Support for ZK circuit integration
 */
interface IAuthorizationVerifier {
    
    // ============ Authorization Types ============
    enum AuthType {
        User,        // 0: Single signature
        Admin,       // 1: M-of-N multisig
        Bridge,      // 2: Bridge attestation
        Governance,  // 3: Governance vote
        System,      // 4: System call
        Emergency    // 5: Emergency key
    }
    
    // ============ Events ============
    event AuthorizationVerified(
        AuthType indexed authType,
        bytes32 indexed commitment,
        address indexed verifier
    );
    
    event SignatureValidated(
        address indexed signer,
        bytes32 indexed messageHash
    );
    
    event MultisigValidated(
        address[] signers,
        uint256 threshold
    );
    
    // ============ Functions ============
    
    /**
     * @notice Verify authorization based on type
     * @param authType Type of authorization
     * @param data Encoded authorization data (signatures, commitments, etc)
     * @return valid Whether authorization is valid
     */
    function verifyAuthorization(
        AuthType authType,
        bytes calldata data
    ) external view returns (bool valid);
    
    /**
     * @notice Get commitment to authorization proof
     * @param authType Type of authorization
     * @param data Encoded authorization data
     * @return commitment Keccak256 hash of (authType, data)
     * 
     * Used by ZK circuit to verify authorization without revealing details
     */
    function getAuthorizationCommitment(
        AuthType authType,
        bytes calldata data
    ) external view returns (bytes32 commitment);
    
    /**
     * @notice Verify user signature
     * @param messageHash Message that was signed
     * @param signature Raw signature bytes (65 bytes)
     * @param expectedSigner Expected signer address
     * @return valid Whether signature is valid
     */
    function verifyUserSignature(
        bytes32 messageHash,
        bytes calldata signature,
        address expectedSigner
    ) external view returns (bool valid);
    
    /**
     * @notice Verify admin multisig
     * @param messageHash Message that was signed
     * @param signatures Array of signatures (65 bytes each)
     * @param signers Array of signer addresses
     * @param threshold Minimum signatures required
     * @return valid Whether multisig is valid
     */
    function verifyAdminMultisig(
        bytes32 messageHash,
        bytes[] calldata signatures,
        address[] calldata signers,
        uint256 threshold
    ) external view returns (bool valid);
    
    /**
     * @notice Verify governance proposal
     * @param yesVotes Number of yes votes
     * @param noVotes Number of no votes
     * @param threshold Minimum vote threshold
     * @return valid Whether proposal passed
     */
    function verifyGovernanceProposal(
        uint256 yesVotes,
        uint256 noVotes,
        uint256 threshold
    ) external view returns (bool valid);
}