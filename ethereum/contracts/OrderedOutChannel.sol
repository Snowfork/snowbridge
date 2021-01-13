 // SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "./ChannelOut.sol";

// OrderedOutChannel is a basic channel that just outputs messages with an increasing nonce.
contract OrderedOutChannel is ChannelOut  {

    uint256 public currentNonce;

    event NewMessage(uint256 nonce, address senderAddress, string targetApplicationId, bytes payload);

    constructor() public {
        currentNonce = 0;
    }

    /**
     * @dev Submits a message into the channel
     */
    function submit(string memory targetApplicationId, bytes memory payload) public override {
        emit NewMessage(currentNonce, msg.sender, targetApplicationId, payload);
        currentNonce++;
    }

}
