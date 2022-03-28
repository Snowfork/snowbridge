// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "../BeefyLightClient.sol";
import "../ValidatorRegistry.sol";
import "../SimplifiedMMRVerification.sol";

contract ExposedBeefyLightClient is BeefyLightClient {

    constructor() BeefyLightClient(ValidatorRegistry(address(0)), SimplifiedMMRVerification(address(0)), 0) {}

    function encodeCommitmentExposed(Commitment calldata commitment)
        external
        pure
        returns (bytes memory)
    {
        return encodeCommitment(commitment);
    }
}
