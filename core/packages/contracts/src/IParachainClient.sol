// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

interface IParachainClient {
    function verifyCommitment(bytes32 commitment, bytes calldata opaqueProof) external view returns (bool);
}
