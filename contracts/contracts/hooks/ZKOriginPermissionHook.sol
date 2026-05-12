// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IZKOriginHook} from "../interfaces/IZKOriginHook.sol";

interface IPermissionVerifier {
    function verifyProof(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[8] calldata pubSignals
    ) external view returns (bool);
}

contract ZKOriginPermissionHook is IZKOriginHook {

    uint8 public constant ACTION_SWAP = 0;
    uint8 public constant ACTION_ADD_LIQUIDITY = 1;
    uint8 public constant ACTION_REMOVE_LIQUIDITY = 2;
    uint8 public constant ACTION_CHANGE_FEE = 3;

    uint8 public constant REQUIRED_SWAP = 1;
    uint8 public constant REQUIRED_ADD_LIQUIDITY = 1;
    uint8 public constant REQUIRED_REMOVE_LIQUIDITY = 1;
    uint8 public constant REQUIRED_CHANGE_FEE = 4;

    IPermissionVerifier public immutable permissionVerifier;
    address public admin;

    mapping(bytes32 => mapping(address => LineageState)) public callerPermissions;
    mapping(bytes32 => bool) public usedPermissionProofs;
    mapping(bytes32 => mapping(uint8 => uint8)) public poolActionRequirements;

    error InvalidProof();
    error ProofAlreadyUsed();
    error PermissionDeniedNoProof();
    error InvalidActionType();
    error ZeroAddress();
    error CallerNotAuthorized();

    event PoolActionRequirementSet(
        bytes32 indexed poolId,
        uint8 actionType,
        uint8 requiredOriginClass
    );

    constructor(address _permissionVerifier) {
        if (_permissionVerifier == address(0)) revert ZeroAddress();
        permissionVerifier = IPermissionVerifier(_permissionVerifier);
        admin = msg.sender;
    }

    function _verifyPermission(
        address sender,
        bytes32 poolId,
        uint8 actionType,
        bytes calldata hookData
    ) internal {

        if (hookData.length == 0) revert PermissionDeniedNoProof();

        (
            uint256[2] memory pA,
            uint256[2][2] memory pB,
            uint256[2] memory pC,
            uint256[8] memory pubSignals
        ) = abi.decode(hookData, (uint256[2], uint256[2][2], uint256[2], uint256[8]));

        bytes32 proofPoolId = bytes32(pubSignals[1]);
        uint8 proofActionType = uint8(pubSignals[2]);
        uint8 claimedOriginClass = uint8(pubSignals[3]);
        bytes32 lineageCommitment = bytes32(pubSignals[4]);

        require(proofPoolId == poolId, "ZKOriginPermissionHook: pool ID mismatch");
        require(proofActionType == actionType, "ZKOriginPermissionHook: action type mismatch");

        uint8 required = poolActionRequirements[poolId][actionType];
        require(
            claimedOriginClass >= required,
            "ZKOriginPermissionHook: insufficient origin class"
        );

        bytes32 proofHash = keccak256(abi.encode(pA, pB, pC, pubSignals));
        if (usedPermissionProofs[proofHash]) revert ProofAlreadyUsed();
        usedPermissionProofs[proofHash] = true;

        bool proofValid = permissionVerifier.verifyProof(pA, pB, pC, pubSignals);
        if (!proofValid) {
            emit PermissionDenied(sender, poolId, "ZK proof invalid");
            revert InvalidProof();
        }

        callerPermissions[poolId][sender] = LineageState({
            lineageCommitment: lineageCommitment,
            originClass: claimedOriginClass,
            depth: callerPermissions[poolId][sender].depth + 1,
            epoch: pubSignals[6],
            verified: true
        });

        emit PermissionGranted(sender, poolId, claimedOriginClass);
    }

    function checkSwapPermission(
        address sender,
        bytes32 poolId,
        bytes calldata hookData
    ) external {
        _verifyPermission(sender, poolId, ACTION_SWAP, hookData);
    }

    function checkAddLiquidityPermission(
        address sender,
        bytes32 poolId,
        bytes calldata hookData
    ) external {
        _verifyPermission(sender, poolId, ACTION_ADD_LIQUIDITY, hookData);
    }

    function checkRemoveLiquidityPermission(
        address sender,
        bytes32 poolId,
        bytes calldata hookData
    ) external {
        _verifyPermission(sender, poolId, ACTION_REMOVE_LIQUIDITY, hookData);
    }

    function checkChangeFeePerm(
        address sender,
        bytes32 poolId,
        bytes calldata hookData
    ) external {
        _verifyPermission(sender, poolId, ACTION_CHANGE_FEE, hookData);
    }

    function setPoolActionRequirement(
        bytes32 poolId,
        uint8 actionType,
        uint8 requiredOriginClass
    ) external {
        require(msg.sender == admin, "ZKOriginPermissionHook: not admin");
        if (actionType >= 4) revert InvalidActionType();
        if (requiredOriginClass >= 7) revert InvalidActionType();

        poolActionRequirements[poolId][actionType] = requiredOriginClass;
        emit PoolActionRequirementSet(poolId, actionType, requiredOriginClass);
    }

    function getCallerPermission(bytes32 poolId, address caller)
        external view returns (LineageState memory)
    {
        return callerPermissions[poolId][caller];
    }

    function getActionRequirement(bytes32 poolId, uint8 actionType)
        external view returns (uint8)
    {
        return poolActionRequirements[poolId][actionType];
    }

    function initializePoolRequirements(bytes32 poolId) external {
        require(msg.sender == admin, "ZKOriginPermissionHook: not admin");
        poolActionRequirements[poolId][ACTION_SWAP] = REQUIRED_SWAP;
        poolActionRequirements[poolId][ACTION_ADD_LIQUIDITY] = REQUIRED_ADD_LIQUIDITY;
        poolActionRequirements[poolId][ACTION_REMOVE_LIQUIDITY] = REQUIRED_REMOVE_LIQUIDITY;
        poolActionRequirements[poolId][ACTION_CHANGE_FEE] = REQUIRED_CHANGE_FEE;
    }

    function transferAdmin(address newAdmin) external {
        require(msg.sender == admin, "ZKOriginPermissionHook: not admin");
        if (newAdmin == address(0)) revert ZeroAddress();
        admin = newAdmin;
    }
}