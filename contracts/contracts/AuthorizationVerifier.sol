// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/IAuthorizationVerifier.sol";

/**
 * @title AuthorizationVerifier (PRODUCTION)
 * @notice Verifies authorization proofs for all origin classes
 */

contract AuthorizationVerifier is IAuthorizationVerifier {
    
    // ============ Events (Custom, non-interface) ============
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
    error InvalidProposal();
    error FinalityNotReached();
    error QuorumNotMet();
    
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
        require(msg.sender == admin, "NotAdmin");
        _;
    }
    
    // ============ Constructor ============
    constructor() {
        admin = msg.sender;
    }
    
    // ============ Signature Recovery ============
    
    /**
     * @notice Recover signer from signature with ALL security checks
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
        
        // SECURITY FIX 1: Don't auto-correct v
        if (v != 27 && v != 28) {
            return address(0);
        }
        
        // SECURITY FIX 2: Check for malleability
        if (uint256(s) > SECP256K1_N_DIV_2) {
            return address(0);
        }
        
        // Recover signer
        signer = ecrecover(messageHash, v, r, s);
        
        // SECURITY FIX 3: Check for recovery failure
        if (signer == address(0)) {
            return address(0);
        }
        
        return signer;
    }
    
    // ============ User Authorization ============
    
    /**
     * @notice Verify user signature
     */
    function verifyUserSignature(
        bytes32 messageHash,
        bytes memory signature,
        address expectedSigner
    ) public pure returns (bool valid) {
        require(expectedSigner != address(0), "ZeroAddress");
        require(messageHash != bytes32(0), "InvalidProof");
        
        // Create Ethereum signed message hash
        bytes32 ethSignedHash = keccak256(
            abi.encodePacked(
                "\x19Ethereum Signed Message:\n32",
                messageHash
            )
        );
        
        // Recover signer
        address recoveredSigner = recoverSigner(ethSignedHash, signature);
        
        // Check it matches expected
        valid = (recoveredSigner == expectedSigner);
    }
    
    // ============ Admin Authorization ============
    
    /**
     * @notice Verify M-of-N multisig
     */
    function verifyAdminMultisig(
        bytes32 messageHash,
        bytes[] memory signatures,
        address[] memory signers,
        uint256 threshold
    ) public pure returns (bool valid) {
        // Validate inputs
        require(signatures.length == signers.length, "LengthMismatch");
        require(signatures.length <= MAX_SIGNERS, "TooManySigners");
        require(threshold >= MIN_THRESHOLD, "ThresholdTooLow");
        require(threshold <= signatures.length, "ThresholdTooHigh");
        
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
            // Recover signer
            address recoveredSigner = recoverSigner(ethSignedHash, signatures[i]);
            
            // Must match expected signer
            require(recoveredSigner == signers[i], "InvalidSignature");
            
            // Check for duplicate signers
            for (uint256 j = i + 1; j < signers.length; j++) {
                require(signers[i] != signers[j], "DuplicateSigner");
            }
            
            validSignatures++;
        }
        
        // Check threshold met
        valid = (validSignatures >= threshold);
    }
    
    // ============ Governance Authorization ============
    
    /**
     * @notice Verify governance proposal passed
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
            
        } else if (authType == AuthType.Bridge) {
            (uint256 sourceChainId, bytes32 stateRoot, bytes memory signature, address bridgeKey) = 
                abi.decode(data, (uint256, bytes32, bytes, address));
            
            valid = verifyBridgeAttestation(sourceChainId, stateRoot, signature, bridgeKey);
            
        } else if (authType == AuthType.Governance) {
            (uint256 yesVotes, uint256 noVotes, uint256 threshold) = 
                abi.decode(data, (uint256, uint256, uint256));
            
            valid = verifyGovernanceProposal(yesVotes, noVotes, threshold);
            
        } else if (authType == AuthType.System) {
            (address callerAddress, address expectedSystemAddress) = 
                abi.decode(data, (address, address));
            
            valid = (callerAddress == expectedSystemAddress);
            
        } else if (authType == AuthType.Emergency) {
            (bytes32 emergencyKeyHash, address actualKey) = 
                abi.decode(data, (bytes32, address));
            
            valid = (keccak256(abi.encodePacked(actualKey)) == emergencyKeyHash);
        } else {
            revert("InvalidAuthType");
        }
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Approve a signer
     */
    function approveSigner(address signer) external onlyAdmin {
        require(signer != address(0), "ZeroAddress");
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
        require(newAdmin != address(0), "ZeroAddress");
        admin = newAdmin;
    }
}