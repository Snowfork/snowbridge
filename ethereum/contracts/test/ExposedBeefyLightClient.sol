// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "../BeefyClient.sol";
import "../utils/MMRProofVerification.sol";

contract ExposedBeefyLightClient is BeefyClient {

    constructor() BeefyClient(SimplifiedMMRVerification(address(0))) {}

    function encodeCommitmentExposed(Commitment calldata commitment)
        external
        pure
        returns (bytes memory)
    {
        return encodeCommitment(commitment);
    }
}
