
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IZKOriginHook {

    struct HookProof {
        uint256[2] pA;
        uint256[2][2] pB;
        uint256[2] pC;
        uint256[12] publicSignals;
        uint8 authType;
        bytes authData;
    }

    struct LineageState {
        bytes32 lineageCommitment;
        uint8 originClass;
        uint256 depth;
        uint256 epoch;
        bool verified;
    }

    event LineageProved(
        bytes32 indexed poolId,
        bytes32 indexed lineageCommitment,
        uint8 originClass,
        address caller
    );

    event LineageRejected(
        bytes32 indexed poolId,
        string reason
    );

    event PermissionGranted(
        address indexed caller,
        bytes32 indexed poolId,
        uint8 originClass
    );

    event PermissionDenied(
        address indexed caller,
        bytes32 indexed poolId,
        string reason
    );
}