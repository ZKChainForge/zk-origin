
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract MockGroth16Verifier {
    bool public shouldVerify = true;
    
    function setVerifyResult(bool _result) external {
        shouldVerify = _result;
    }
    
    function verifyProof(
        uint256[2] calldata,
        uint256[2][2] calldata,
        uint256[2] calldata,
        uint256[2] calldata
    ) external view returns (bool) {
        return shouldVerify;
    }
}
