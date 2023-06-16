// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

interface IParachainClient {
    function verifyCommitment(bytes32 commitment, bytes calldata opaqueProof) external view returns (bool);
}
