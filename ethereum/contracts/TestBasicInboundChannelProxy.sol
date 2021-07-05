// SPDX-License-Identifier: MIT
pragma solidity ^0.8.6;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./ParachainLightClient.sol";
import "./BasicInboundChannel.sol";

contract TestBasicInboundChannelProxy is Ownable {
    BasicInboundChannel channel;

    constructor(address _channel) {
        channel = BasicInboundChannel(_channel);
    }

    function submit(
        BasicInboundChannel.Message[] calldata _messages,
        ParachainLightClient.OwnParachainHeadPartial
            calldata _ownParachainHeadPartial,
        ParachainLightClient.ParachainHeadProof calldata _parachainHeadProof,
        ParachainLightClient.BeefyMMRLeafPartial calldata _beefyMMRLeafPartial,
        uint256 _beefyMMRLeafIndex,
        uint256 _beefyMMRLeafCount,
        bytes32[] calldata _beefyMMRLeafProof
    ) public {
        channel.submit(
            _messages,
            _ownParachainHeadPartial,
            _parachainHeadProof,
            _beefyMMRLeafPartial,
            _beefyMMRLeafIndex,
            _beefyMMRLeafCount,
            _beefyMMRLeafProof
        );
    }
}
