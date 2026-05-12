// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IZKOriginHook} from "../interfaces/IZKOriginHook.sol";

interface IDonationVerifier {
    function verifyProof(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[12] calldata pubSignals
    ) external view returns (bool);
}

interface ILineageVerifierHook {
    function hasVerifiedLineage(bytes32 stateHash) external view returns (bool);
    function getDepth(bytes32 stateHash) external view returns (uint256);
}

contract ZKOriginDonationHook is IZKOriginHook {

    IDonationVerifier public immutable donationVerifier;
    ILineageVerifierHook public immutable lineageVerifier;

    address public admin;

    mapping(bytes32 => LineageState) public poolLineage;
    mapping(bytes32 => mapping(bytes32 => bool)) public usedProofs;
    mapping(bytes32 => uint256) public totalDonations;
    mapping(bytes32 => uint256) public donationCount;

    error InvalidProof();
    error ProofAlreadyUsed();
    error LineageNotVerified();
    error InvalidDonationAmount();
    error UnauthorizedCaller();
    error ZeroAddress();
    error PoolNotInitialized();

    constructor(
        address _donationVerifier,
        address _lineageVerifier
    ) {
        if (_donationVerifier == address(0)) revert ZeroAddress();
        if (_lineageVerifier == address(0)) revert ZeroAddress();

        donationVerifier = IDonationVerifier(_donationVerifier);
        lineageVerifier = ILineageVerifierHook(_lineageVerifier);
        admin = msg.sender;
    }

    function beforeDonate(
        address sender,
        bytes32 poolId,
        uint256 amount0,
        uint256 amount1,
        bytes calldata hookData
    ) external returns (bytes4) {

        require(hookData.length > 0, "ZKOriginDonationHook: no proof provided");

        (
            uint256[2] memory pA,
            uint256[2][2] memory pB,
            uint256[2] memory pC,
            uint256[12] memory pubSignals
        ) = abi.decode(hookData, (uint256[2], uint256[2][2], uint256[2], uint256[12]));

        bytes32 proofPoolId = bytes32(pubSignals[0]);
        uint256 donationAmount = pubSignals[1];
        bytes32 newLineageCommitment = bytes32(pubSignals[5]);
        uint8 originClass = 1;

        require(
            proofPoolId == poolId,
            "ZKOriginDonationHook: pool ID mismatch"
        );

        if (donationAmount == 0) revert InvalidDonationAmount();
        if (amount0 == 0 && amount1 == 0) revert InvalidDonationAmount();

        bytes32 proofHash = keccak256(abi.encode(pA, pB, pC, pubSignals));
        if (usedProofs[poolId][proofHash]) revert ProofAlreadyUsed();
        usedProofs[poolId][proofHash] = true;

        bool proofValid = donationVerifier.verifyProof(pA, pB, pC, pubSignals);
        if (!proofValid) {
            emit LineageRejected(poolId, "ZK proof invalid");
            revert InvalidProof();
        }

        poolLineage[poolId] = LineageState({
            lineageCommitment: newLineageCommitment,
            originClass: originClass,
            depth: poolLineage[poolId].depth + 1,
            epoch: pubSignals[9],
            verified: true
        });

        totalDonations[poolId] += donationAmount;
        donationCount[poolId]++;

        emit LineageProved(poolId, newLineageCommitment, originClass, sender);

        return this.beforeDonate.selector;
    }

    function afterDonate(
        address sender,
        bytes32 poolId,
        uint256 amount0,
        uint256 amount1,
        bytes calldata hookData
    ) external returns (bytes4) {

        require(poolLineage[poolId].verified, "ZKOriginDonationHook: lineage not recorded");

        return this.afterDonate.selector;
    }

    function getPoolLineage(bytes32 poolId)
        external view returns (LineageState memory)
    {
        return poolLineage[poolId];
    }

    function isDonationVerified(bytes32 poolId)
        external view returns (bool)
    {
        return poolLineage[poolId].verified;
    }

    function getPoolStats(bytes32 poolId)
        external view returns (uint256 total, uint256 count)
    {
        return (totalDonations[poolId], donationCount[poolId]);
    }

    function transferAdmin(address newAdmin) external {
        require(msg.sender == admin, "ZKOriginDonationHook: not admin");
        if (newAdmin == address(0)) revert ZeroAddress();
        admin = newAdmin;
    }
}