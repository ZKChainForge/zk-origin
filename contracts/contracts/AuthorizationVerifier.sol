// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/IAuthorizationVerifier.sol";

/**
 * @title AuthorizationVerifier
 * @notice Verifies authorization proofs for different origin classes
 */
contract AuthorizationVerifier is IAuthorizationVerifier {
    
    // ============ Events ============
    event AuthorizationVerified(AuthType indexed authType, bytes32 commitment);
    event UserAuthVerified(address indexed user, bytes32 messageHash);
    event AdminAuthVerified(address[] signers, uint256 threshold);
    event GovernanceAuthVerified(uint256 indexed proposalId, uint256 votes);
    
    // ============ Errors ============
    error InvalidAuthType();
    error InvalidSignature();
    error ThresholdNotMet();
    error InvalidProof();
    error ZeroAddress();
    
    // ============ Constants ============
    uint256 public constant MAX_SIGNERS = 15;
    uint256 public constant MIN_THRESHOLD = 1;
    
    // ============ State ============
    address public admin;
    
    mapping(address => bool) public approvedSigners;
    mapping(uint256 => bool) public usedNonces;
    
    // ============ Modifiers ============
    modifier onlyAdmin() {
        if (msg.sender != admin) revert();
        _;
    }
    
    // ============ Constructor ============
    constructor() {
        admin = msg.sender;
    }
    
    // ============ User Authorization ============
    
    function verifyUserSignature(
        bytes32 messageHash,
        bytes calldata signature,
        address signer
    ) public pure returns (bool valid) {
        if (signer == address(0)) revert ZeroAddress();
        
        bytes32 ethSignedHash = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash)
        );
        
        address recoveredSigner = recoverSigner(ethSignedHash, signature);
        valid = (recoveredSigner == signer);
    }
    
    function recoverSigner(
        bytes32 messageHash,
        bytes calldata signature
    ) internal pure returns (address) {
        if (signature.length != 65) return address(0);
        
        bytes32 r;
        bytes32 s;
        uint8 v;
        
        assembly {
            r := calldataload(add(signature.offset, 0))
            s := calldataload(add(signature.offset, 32))
            v := byte(0, calldataload(add(signature.offset, 64)))
        }
        
        if (v < 27) v += 27;
        if (v != 27 && v != 28) return address(0);
        
        address recovered = ecrecover(messageHash, v, r, s);
        return recovered;
    }
    
    // ============ Admin Authorization ============
    
    function verifyAdminMultisigMemory(
        bytes32 messageHash,
        bytes[] memory signatures,
        address[] memory signers,
        uint256 threshold
    ) internal pure returns (bool valid) {
        if (signatures.length != signers.length) return false;
        if (signatures.length > MAX_SIGNERS) return false;
        if (threshold < MIN_THRESHOLD) return false;
        if (threshold > signatures.length) return false;
        
        uint256 validSignatures = 0;
        bytes32 ethSignedHash = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash)
        );
        
        for (uint256 i = 0; i < signatures.length; i++) {
            address recoveredSigner = recoverSignerMemory(ethSignedHash, signatures[i]);
            
            if (recoveredSigner == signers[i]) {
                validSignatures++;
            }
            
            for (uint256 j = i + 1; j < signers.length; j++) {
                if (signers[i] == signers[j]) return false;
            }
        }
        
        valid = (validSignatures >= threshold);
    }
    
    function recoverSignerMemory(
        bytes32 messageHash,
        bytes memory signature
    ) internal pure returns (address) {
        if (signature.length != 65) return address(0);
        
        bytes32 r;
        bytes32 s;
        uint8 v;
        
        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }
        
        if (v < 27) v += 27;
        if (v != 27 && v != 28) return address(0);
        
        address recovered = ecrecover(messageHash, v, r, s);
        return recovered;
    }
    
    // ============ Governance Authorization ============
    
    function verifyGovernanceProposal(
        uint256 yesVotes,
        uint256 /* noVotes */,
        uint256 threshold
    ) public pure returns (bool valid) {
        valid = (yesVotes > threshold);
    }
    
    // ============ Bridge Authorization ============
    
    function verifyBridgeAttestation(
        uint256 sourceChainId,
        bytes32 stateRoot,
        bytes calldata signature,
        address bridgeKey
    ) public pure returns (bool valid) {
        bytes32 messageHash = keccak256(
            abi.encodePacked(sourceChainId, stateRoot)
        );
        
        valid = verifyUserSignature(messageHash, signature, bridgeKey);
    }
    
    // ============ Authorization Verification (Main Interface) ============
    
    function verifyAuthorization(
        AuthType authType,
        bytes calldata data
    ) external pure override returns (bool valid) {
        if (authType == AuthType.User) {
            (bytes32 messageHash, bytes memory signature, address signer) = abi.decode(
                data,
                (bytes32, bytes, address)
            );
            
            bytes memory sigMem = signature;
            if (sigMem.length != 65) return false;
            
            bytes32 ethSignedHash = keccak256(
                abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash)
            );
            address recoveredSigner = recoverSignerMemory(ethSignedHash, sigMem);
            return recoveredSigner == signer;
            
        } else if (authType == AuthType.Admin) {
            (bytes32 messageHash, bytes[] memory signatures, address[] memory signers, uint256 threshold) = abi.decode(
                data,
                (bytes32, bytes[], address[], uint256)
            );
            return verifyAdminMultisigMemory(messageHash, signatures, signers, threshold);
            
        } else if (authType == AuthType.Governance) {
            (uint256 yesVotes, uint256 noVotes, uint256 threshold) = abi.decode(
                data,
                (uint256, uint256, uint256)
            );
            return verifyGovernanceProposal(yesVotes, noVotes, threshold);
            
        } else if (authType == AuthType.Bridge) {
            (uint256 sourceChainId, bytes32 stateRoot, bytes memory signature, address bridgeKey) = abi.decode(
                data,
                (uint256, bytes32, bytes, address)
            );
            
            bytes32 messageHash = keccak256(
                abi.encodePacked(sourceChainId, stateRoot)
            );
            bytes32 ethSignedHash = keccak256(
                abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash)
            );
            address recoveredSigner = recoverSignerMemory(ethSignedHash, signature);
            return recoveredSigner == bridgeKey;
            
        } else {
            revert InvalidAuthType();
        }
    }
    
    function getAuthorizationCommitment(
        AuthType authType,
        bytes calldata data
    ) external pure override returns (bytes32 commitment) {
        return keccak256(abi.encodePacked(authType, data));
    }
}