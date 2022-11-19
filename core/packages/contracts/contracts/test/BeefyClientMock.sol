// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../BeefyClient.sol";

contract BeefyClientMock is BeefyClient {
    constructor() BeefyClient() {}

    // Make seedFromPrevRandao return the same result over multiple test runs
    function seedFromPrevRandao(uint256) internal override pure returns (uint256) {
        return 377;
    }

    function encodeCommitment_public(Commitment calldata commitment)
        external
        pure
        returns (bytes memory)
    {
        return encodeCommitment(commitment);
    }
}
