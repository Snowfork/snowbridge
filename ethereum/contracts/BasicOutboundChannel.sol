// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./OutboundChannel.sol";

// BasicOutboundChannel is a basic channel that just sends messages with a nonce.
contract BasicOutboundChannel is OutboundChannel {
    using SafeMath for uint64;

    mapping(address => uint64) account_nonces;

    event Message(
        address origin,
        address source,
        uint64  nonce,
        bytes   payload
    );

    /**
     * @dev Sends a message across the channel
     */
    function submit(address origin, bytes memory payload) public override {
        emit Message(origin, msg.sender, ++account_nonces[origin], payload);
    }
}
