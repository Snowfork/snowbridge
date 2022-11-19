// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../BeefyClient.sol";

contract BeefyClientMock is BeefyClient {
    constructor() BeefyClient() {}

    // Make deriveSeed return the same result over multiple test runs
    function deriveSeed(Request storage) internal override pure returns (uint256) {
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
