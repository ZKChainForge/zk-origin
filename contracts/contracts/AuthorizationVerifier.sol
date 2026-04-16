// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/IAuthorizationVerifier.sol";

/**
 * @title AuthorizationVerifier
 * @notice Verifies authorization proofs for different origin classes
 * 
 * FIXED:
 * - Use memory instead of calldata in internal functions
 * - Proper signature recovery
 * - Malleability prevention
 */
contract AuthorizationVerifier is IAuthorizationVerifier {
    
    // ============ Events ============
    event AuthorizationVerified(AuthType indexed authType, bytes32 commitment);
    event UserAuthVerified(address indexed user, bytes32 messageHash);
    event AdminAuthVerified(address[] signers, uint256 threshold);
    event GovernanceAuthVerified(uint256 indexed proposalId, uint256 votes);
    event SignatureMalleabilityDetected(bytes32 messageHash);
    
    // ============ Errors ============
    error InvalidAuthType();
    error InvalidSignature();
    error ThresholdNotMet();
    error InvalidProof();
    error ZeroAddress();
    error MalleableSignature();
    error RecoveryFailed();
    error DuplicateSigner();
    
    // ============ Constants ============
    uint256 public constant MAX_SIGNERS = 15;
    uint256 public constant MIN_THRESHOLD = 1;
    
    // secp256k1 order / 2 for malleability check
    uint256 private constant SECP256K1_N_DIV_2 = 
        0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0;
    
    // ============ State ============
    address public admin;
    mapping(address => bool) public approvedSigners;
    mapping(bytes32 => bool) public usedNonces;
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        if (msg.sender != admin) revert();
        _;
    }
    
    // ============ Constructor ============
    constructor() {
        admin = msg.sender;
    }
    
    // ============ Signature Recovery (FIXED) ============
    
    /**
     * @notice Recover signer from signature with ALL security checks
     * 
     * SECURITY:
     * 1. Validates v is 27 or 28 (no auto-correction)
     * 2. Prevents malleable signatures (checks s <= n/2)
     * 3. Checks for address(0) recovery failure
     * 4. Explicit return on any failure
     */
    function recoverSigner(
        bytes32 messageHash,
        bytes memory signature
    ) internal pure returns (address signer) {
        // Validate signature length
        if (signature.length != 65) {
            return address(0);
        }
        
        bytes32 r;
        bytes32 s;
        uint8 v;
        
        // Extract r, s, v from signature
        assembly {
            r := mload(add(signature, 0x20))
            s := mload(add(signature, 0x40))
            v := byte(0, mload(add(signature, 0x60)))
        }
        
        // ✅ FIX 1: Don't auto-correct v
        // v MUST be exactly 27 or 28
        if (v != 27 && v != 28) {
            return address(0);
        }
        
        // ✅ FIX 2: Check for malleability
        // If s > n/2, signature is malleable
        if (uint256(s) > SECP256K1_N_DIV_2) {
            return address(0);
        }
        
        // Recover signer using ecrecover
        signer = ecrecover(messageHash, v, r, s);
        
        // ✅ FIX 3: Explicit check for recovery failure
        if (signer == address(0)) {
            return address(0);
        }
        
        return signer;
    }
    
    // ============ User Authorization ============
    
    /**
     * @notice Verify user signature
     * @param messageHash Hash of message that was signed
     * @param signature Raw signature (65 bytes)
     * @param expectedSigner Expected signer address
     * @return valid Whether signature is valid
     */
    function verifyUserSignature(
        bytes32 messageHash,
        bytes memory signature,
        address expectedSigner
    ) public pure returns (bool valid) {
        if (expectedSigner == address(0)) revert ZeroAddress();
        if (messageHash == bytes32(0)) revert InvalidProof();
        
        // Create Ethereum signed message hash
        bytes32 ethSignedHash = keccak256(
            abi.encodePacked(
                "\x19Ethereum Signed Message:\n32",
                messageHash
            )
        );
        
        // Recover signer (with all safety checks)
        address recoveredSigner = recoverSigner(ethSignedHash, signature);
        
        // Check it matches expected
        valid = (recoveredSigner == expectedSigner);
    }
    
    // ============ Admin Authorization (Multisig) ============
    
    /**
     * @notice Verify M-of-N multisig
     * @param messageHash Hash of transaction
     * @param signatures Array of signatures
     * @param signers Array of signer addresses
     * @param threshold Minimum valid signatures required
     * @return valid Whether multisig is valid
     */
    function verifyAdminMultisig(
        bytes32 messageHash,
        bytes[] memory signatures,
        address[] memory signers,
        uint256 threshold
    ) public pure returns (bool valid) {
        // Validate inputs
        if (signatures.length != signers.length) return false;
        if (signatures.length > MAX_SIGNERS) return false;
        if (threshold < MIN_THRESHOLD) return false;
        if (threshold > signatures.length) return false;
        
        // Create Ethereum signed message hash
        bytes32 ethSignedHash = keccak256(
            abi.encodePacked(
                "\x19Ethereum Signed Message:\n32",
                messageHash
            )
        );
        
        uint256 validSignatures = 0;
        
        // Verify each signature
        for (uint256 i = 0; i < signatures.length; i++) {
            // Recover signer (with safety checks)
            address recoveredSigner = recoverSigner(ethSignedHash, signatures[i]);
            
            // Must match expected signer
            if (recoveredSigner != signers[i]) {
                return false;
            }
            
            // Check for duplicate signers
            for (uint256 j = i + 1; j < signers.length; j++) {
                if (signers[i] == signers[j]) {
                    revert DuplicateSigner();
                }
            }
            
            validSignatures++;
        }
        
        // Check threshold met
        valid = (validSignatures >= threshold);
    }
    
    // ============ Governance Authorization ============
    
    /**
     * @notice Verify governance proposal passed
     * @param yesVotes Number of yes votes
     * @param noVotes Number of no votes
     * @param threshold Minimum required votes
     * @return valid Whether governance check passes
     */
    function verifyGovernanceProposal(
        uint256 yesVotes,
        uint256 noVotes,
        uint256 threshold
    ) public pure returns (bool valid) {
        // Yes votes must exceed no votes + threshold
        valid = (yesVotes > (noVotes + threshold));
    }
    
    // ============ Bridge Authorization ============
    
    /**
     * @notice Verify bridge attestation
     * @param sourceChainId Chain ID of source
     * @param stateRoot Root of state tree
     * @param signature Bridge signature
     * @param bridgeKey Expected bridge key
     * @return valid Whether attestation is valid
     */
    function verifyBridgeAttestation(
        uint256 sourceChainId,
        bytes32 stateRoot,
        bytes memory signature,
        address bridgeKey
    ) public pure returns (bool valid) {
        // Create message hash
        bytes32 messageHash = keccak256(
            abi.encodePacked(sourceChainId, stateRoot)
        );
        
        // Verify signature
        valid = verifyUserSignature(messageHash, signature, bridgeKey);
    }
    
    // ============ Authorization Commitment ============
    
    /**
     * @notice Compute commitment to authorization
     * @param authType Type of authorization
     * @param data Encoded authorization data
     * @return commitment Hash commitment to auth
     */
    function getAuthorizationCommitment(
        AuthType authType,
        bytes calldata data
    ) external pure override returns (bytes32 commitment) {
        commitment = keccak256(abi.encodePacked(authType, data));
    }
    
    // ============ Main Verification Function ============
    
    /**
     * @notice Verify authorization based on type
     * @param authType Type of authorization (User, Admin, Bridge, etc)
     * @param data Encoded authorization data
     * @return valid Whether authorization is valid
     * 
     * Data encoding:
     * - User: (bytes32 messageHash, bytes signature, address signer)
     * - Admin: (bytes32 messageHash, bytes[] sigs, address[] signers, uint threshold)
     * - Governance: (uint yesVotes, uint noVotes, uint threshold)
     * - Bridge: (uint sourceChainId, bytes32 stateRoot, bytes sig, address bridgeKey)
     */
    function verifyAuthorization(
        AuthType authType,
        bytes calldata data
    ) external pure override returns (bool valid) {
        if (authType == AuthType.User) {
            (bytes32 messageHash, bytes memory signature, address signer) = 
                abi.decode(data, (bytes32, bytes, address));
            
            valid = verifyUserSignature(messageHash, signature, signer);
            
        } else if (authType == AuthType.Admin) {
            (bytes32 messageHash, bytes[] memory signatures, address[] memory signers, uint256 threshold) = 
                abi.decode(data, (bytes32, bytes[], address[], uint256));
            
            valid = verifyAdminMultisig(messageHash, signatures, signers, threshold);
            
        } else if (authType == AuthType.Governance) {
            (uint256 yesVotes, uint256 noVotes, uint256 threshold) = 
                abi.decode(data, (uint256, uint256, uint256));
            
            valid = verifyGovernanceProposal(yesVotes, noVotes, threshold);
            
        } else if (authType == AuthType.Bridge) {
            (uint256 sourceChainId, bytes32 stateRoot, bytes memory signature, address bridgeKey) = 
                abi.decode(data, (uint256, bytes32, bytes, address));
            
            valid = verifyBridgeAttestation(sourceChainId, stateRoot, signature, bridgeKey);
            
        } else {
            revert InvalidAuthType();
        }
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Approve a signer
     */
    function approveSigner(address signer) external onlyAdmin {
        if (signer == address(0)) revert ZeroAddress();
        approvedSigners[signer] = true;
    }
    
    /**
     * @notice Revoke a signer
     */
    function revokeSigner(address signer) external onlyAdmin {
        approvedSigners[signer] = false;
    }
    
    /**
     * @notice Transfer admin
     */
    function transferAdmin(address newAdmin) external onlyAdmin {
        if (newAdmin == address(0)) revert ZeroAddress();
        admin = newAdmin;
    }
}