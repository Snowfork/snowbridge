// SPDX-License-Identifier: MIT
pragma solidity ^0.8.6;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./ParachainLightClient.sol";
import "./BasicInboundChannel.sol";

// This contract is used to proxy the BasicInboundChannel and become its owner
// so that any user can access it without being the owner.
// It is only intended to be used for testing purposes, not in production.
contract BasicInboundChannelProxy is Ownable {
    BasicInboundChannel channel;

    constructor(address _channel) {
        channel = BasicInboundChannel(_channel);
    }

    function nonce() public view returns (uint64) {
        return channel.nonce();
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
