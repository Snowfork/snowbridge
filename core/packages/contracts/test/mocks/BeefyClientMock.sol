// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {BeefyClient} from "../../src/BeefyClient.sol";

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

    function initialize_public(
        uint64 _initialBeefyBlock,
        ValidatorSet calldata _initialValidatorSet,
        ValidatorSet calldata _nextValidatorSet
    ) external {
        latestBeefyBlock = _initialBeefyBlock;
        currentValidatorSet = _initialValidatorSet;
        nextValidatorSet = _nextValidatorSet;
    }
}
