// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../BeefyClient.sol";
import "../utils/MMRProofVerification.sol";

contract BeefyClientPublic is BeefyClient {
    constructor() BeefyClient() {}

    function encodeCommitment_public(Commitment calldata commitment)
        external
        pure
        returns (bytes memory)
    {
        return encodeCommitment(commitment);
    }
}
