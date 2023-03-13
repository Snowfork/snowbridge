// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../src/BeefyClient.sol";

contract BeefyClientMock is BeefyClient {
    constructor(uint256 randaoCommitDelay, uint256 randaoCommitExpiration)
        BeefyClient(randaoCommitDelay, randaoCommitExpiration)
    {}

    function encodeCommitment_public(Commitment calldata commitment) external pure returns (bytes memory) {
        return encodeCommitment(commitment);
    }

    function minimumSignatureThreshold_public(uint256 validatorSetLen) external pure returns (uint256) {
        return minimumSignatureThreshold(validatorSetLen);
    }
}
