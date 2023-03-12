// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../IParachainClient.sol";

contract ParachainClientMock is IParachainClient {
    function verifyCommitment(
        bytes32,
        bytes calldata parachainHeaderProof
    ) external pure override returns (bool) {
        if (keccak256(parachainHeaderProof) == keccak256(bytes("validProof"))) {
            return true;
        } else {
            return false;
        }
    }
}
