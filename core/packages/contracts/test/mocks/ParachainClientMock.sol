// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../src/IParachainClient.sol";

contract ParachainClientMock is IParachainClient {
    function verifyCommitment(bytes32, Proof calldata parachainHeaderProof) external pure override returns (bool) {
        if (parachainHeaderProof.headProof.pos == 0) {
            return true;
        } else {
            return false;
        }
    }
}
