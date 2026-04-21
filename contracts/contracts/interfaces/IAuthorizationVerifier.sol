// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IAuthorizationVerifier
 * @notice Interface for authorization proof verification
 * 
 * SECURITY:
 *  Defines all authorization types
 *  Standard verification interface
 *  Commitment-based proof model
 * 
 * AUTHORIZATION TYPES:
 * 0 = User (EdDSA signature)
 * 1 = Admin (M-of-N multisig)
 * 2 = Bridge (Validator quorum + finality)
 * 3 = Governance (Vote threshold + timelock)
 * 4 = System (Authorized caller)
 * 5 = Emergency (Key + conditions)
 */

interface IAuthorizationVerifier {
    
    // ============ Authorization Types ============
    
    enum AuthType {
        User,        // 0
        Admin,       // 1
        Bridge,      // 2
        Governance,  // 3
        System,      // 4
        Emergency    // 5
    }
    
    // ============ Events ============
    
    event AuthorizationVerified(
        AuthType indexed authType,
        bytes32 indexed commitment,
        address indexed creator
    );
    
    event SignatureValidated(
        address indexed signer,
        bytes32 indexed messageHash
    );
    
    event MultisigValidated(
        address[] signers,
        uint256 threshold
    );
    
    event ProposalApproved(
        uint256 indexed proposalId,
        uint256 yesVotes,
        uint256 noVotes
    );
    
    event BridgeAttestation(
        uint256 indexed sourceChain,
        bytes32 stateRoot,
        uint256 validators
    );
    
    event EmergencyActivated(
        address indexed emergencyKey,
        string reason
    );
    
    // ============ Core Functions ============
    
    /**
     * @notice Verify authorization based on type
     * @param authType Type of authorization
     * @param data Encoded authorization data
     * @return valid Whether authorization is valid
     * 
     * Data encoding depends on authType:
     * - User: (bytes32 messageHash, bytes signature, address signer)
     * - Admin: (bytes32 messageHash, bytes[] sigs, address[] signers, uint threshold)
     * - Bridge: (uint sourceChainId, bytes32 stateRoot, bytes sig, address bridgeKey)
     * - Governance: (uint yesVotes, uint noVotes, uint threshold)
     * - System: (address callerAddress, address expectedSystemAddress)
     * - Emergency: (address emergencyKey, bytes32 keyHash)
     */
    function verifyAuthorization(
        AuthType authType,
        bytes calldata data
    ) external view returns (bool valid);
    
    /**
     * @notice Get commitment to authorization proof
     * @param authType Type of authorization
     * @param data Encoded authorization data
     * @return commitment Hash of (authType, data)
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