// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../IParachainClient.sol";

contract ParachainClientMock is IParachainClient {
    function verifyCommitment(bytes32, bytes calldata) external pure override returns (bool) {
        return true;
    }
}
