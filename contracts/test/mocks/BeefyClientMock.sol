// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {BeefyClient} from "../../src/BeefyClient.sol";
import {Counter} from "../../src/utils/Counter.sol";

contract BeefyClientMock is BeefyClient {
    constructor(uint256 randaoCommitDelay, uint256 randaoCommitExpiration, uint256[] memory counter)
        BeefyClient(
            randaoCommitDelay,
            randaoCommitExpiration,
            0,
            BeefyClient.ValidatorSet(0, 0, 0x0, counter),
            BeefyClient.ValidatorSet(0, 0, 0x0, counter)
        )
    {}

    function encodeCommitment_public(Commitment calldata commitment) external pure returns (bytes memory) {
        return encodeCommitment(commitment);
    }

    function setTicketValidatorSetLen(bytes32 commitmentHash, uint32 validatorSetLen) external {
        tickets[createTicketID(msg.sender, commitmentHash)].validatorSetLen = validatorSetLen;
    }

    function setLatestBeefyBlock(uint32 _latestBeefyBlock) external {
        latestBeefyBlock = _latestBeefyBlock;
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

    function minSignatures_public(uint256 validatorSetLen, uint256 signatureUseCount) public pure returns (uint256) {
        return minSignatures(validatorSetLen, signatureUseCount);
    }
}
