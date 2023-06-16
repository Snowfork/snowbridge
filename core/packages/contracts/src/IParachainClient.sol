// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParachainClient} from "./ParachainClient.sol";

interface IParachainClient {
    function verifyCommitment(bytes32 commitment, bytes calldata opaqueProof) external view returns (bool);
    function verifyCommitmentTest(bytes32 commitment, ParachainClient.Proof memory proof)
        external
        view
        returns (bytes memory);
}
